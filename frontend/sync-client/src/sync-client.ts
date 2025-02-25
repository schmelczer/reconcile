import init from "sync_lib";
import wasmBin from "../../../backend/sync_lib/pkg/sync_lib_bg.wasm";
import type { PersistenceProvider } from "./persistence/persistence";
import { SyncHistory } from "./tracing/sync-history";
import { Logger } from "./tracing/logger";
import type { StoredDatabase } from "./persistence/database";
import { Database } from "./persistence/database";
import type { SyncSettings } from "./persistence/settings";
import { Settings } from "./persistence/settings";
import type { CheckConnectionResult } from "./services/sync-service";
import { SyncService } from "./services/sync-service";
import { Syncer } from "./sync-operations/syncer";
import type { FileSystemOperations } from "./file-operations/filesystem-operations";
import { FileOperations } from "./file-operations/file-operations";

export class SyncClient {
	private remoteListenerIntervalId: NodeJS.Timeout | null = null;

	private constructor(
		private readonly _history: SyncHistory,
		private readonly _settings: Settings,
		private readonly _database: Database,
		private readonly _syncer: Syncer,
		private readonly _syncService: SyncService,
		private readonly _logger: Logger
	) {}

	public get history(): SyncHistory {
		return this._history;
	}

	public get settings(): Settings {
		return this._settings;
	}

	public get syncer(): Syncer {
		return this._syncer;
	}

	public get logger(): Logger {
		return this._logger;
	}

	public get documentCount(): number {
		return this._database.getDocuments().size;
	}

	public set fetchImplementation(fetch: typeof globalThis.fetch) {
		this._syncService.fetchImplementation = fetch;
	}

	public static async create(
		fs: FileSystemOperations,
		persistence: PersistenceProvider<
			Partial<{
				settings: Partial<SyncSettings>;
				database: Partial<StoredDatabase>;
			}>
		>
	): Promise<SyncClient> {
		const logger = new Logger();
		const history = new SyncHistory(logger);
		logger.info("Starting SyncClient");

		await init(
			// eslint-disable-next-line
			(wasmBin as any).default // it is loaded as a base64 string by webpack
		);

		let state = (await persistence.load()) ?? {
			settings: undefined,
			database: undefined
		};

		const database = new Database(
			logger,
			state.database,
			async (data): Promise<void> => {
				state = { ...state, database: data };
				return persistence.save(state);
			}
		);

		const settings = new Settings(
			logger,
			state.settings,
			async (data): Promise<void> => {
				state = { ...state, settings: data };
				return persistence.save(state);
			}
		);

		const syncService = new SyncService(settings, logger);

		const syncer = new Syncer(
			logger,
			database,
			settings,
			syncService,
			new FileOperations(logger, database, fs),
			history
		);

		const client = new SyncClient(
			history,
			settings,
			database,
			syncer,
			syncService,
			logger
		);

		void syncer.scheduleSyncForOfflineChanges();

		client.registerRemoteEventListener(
			settings.getSettings().fetchChangesUpdateIntervalMs
		);

		settings.addOnSettingsChangeHandlers((newSettings, oldSettings) => {
			client.registerRemoteEventListener(
				newSettings.fetchChangesUpdateIntervalMs
			);

			if (!oldSettings.isSyncEnabled && newSettings.isSyncEnabled) {
				syncer
					.scheduleSyncForOfflineChanges()
					.catch((_error: unknown) => {
						logger.error(
							"Failed to schedule sync for offline changes"
						);
					});
			}
		});

		logger.info("SyncClient loaded");

		return client;
	}

	public async checkConnection(): Promise<CheckConnectionResult> {
		return this._syncService.checkConnection();
	}

	/// Wait for the in-flight operations to finish, reset all tracking,
	/// and the local database but retain the settings.
	/// The SyncClient can be used again after calling this method.
	public async reset(): Promise<void> {
		this.stop();
		await this._syncer.reset();
		this._history.reset();
		await this._database.resetSyncState();
		this.logger.reset();
	}

	/// Clear all global state that has been touched by SyncClient.
	public stop(): void {
		if (this.remoteListenerIntervalId !== null) {
			clearInterval(this.remoteListenerIntervalId);
		}
	}

	private registerRemoteEventListener(intervalMs: number): void {
		if (this.remoteListenerIntervalId !== null) {
			clearInterval(this.remoteListenerIntervalId);
		}

		this.remoteListenerIntervalId = setInterval(
			() => void this._syncer.applyRemoteChangesLocally(),
			intervalMs
		);
	}
}
