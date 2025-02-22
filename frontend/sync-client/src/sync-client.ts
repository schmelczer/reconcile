import init from "sync_lib";
import wasmBin from "sync_lib/sync_lib_bg.wasm";
import type { PersistenceProvider } from "./persistence/persistence";
import { SyncHistory } from "./tracing/sync-history";
import { Logger } from "./tracing/logger";
import { Database } from "./persistence/database";
import { Settings } from "./persistence/settings";
import type { CheckConnectionResult } from "./services/sync-service";
import { SyncService } from "./services/sync-service";
import { Syncer } from "./sync-operations/syncer";
import { FileSystemOperations } from "./file-operations/filesystem-operations";
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

	public static async create(
		fs: FileSystemOperations,
		persistence: PersistenceProvider
	): Promise<SyncClient> {
		const logger = new Logger();
		const history = new SyncHistory(logger);
		logger.info("Starting SyncClient");

		await init(
			// eslint-disable-next-line
			(wasmBin as any).default // it is loaded as a base64 string by webpack
		);

		let state: Partial<{
			settings: any;
			database: any;
		}> = (await persistence.load()) ?? {
			settings: undefined,
			database: undefined
		};
		const database = new Database(
			logger,
			state.database,
			async (data: unknown): Promise<void> => {
				state = { ...state, database: data };
				return persistence.save(state);
			}
		);

		const settings = new Settings(
			logger,
			state.settings,
			async (data: unknown): Promise<void> => {
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
			new FileOperations(logger, fs),
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

	public get documentCount(): number {
		return this._database.getDocuments().size;
	}

	public async checkConnection(): Promise<CheckConnectionResult> {
		return this._syncService.checkConnection();
	}

	public async reset(): Promise<void> {
		await this._syncer.reset();
		this._history.reset();
		this.logger.reset();
	}

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
