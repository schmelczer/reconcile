import initWasm from "sync_lib";
import wasmBin from "../../../backend/sync_lib/pkg/sync_lib_bg.wasm";
import type { PersistenceProvider } from "./persistence/persistence";
import {
	HistoryEntry,
	HistoryStats,
	SyncHistory
} from "./tracing/sync-history";
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
import { ConnectionStatus } from "./services/connection-status";

export class SyncClient {
	private remoteListenerIntervalId: NodeJS.Timeout | null = null;

	private constructor(
		private readonly _history: SyncHistory,
		private readonly _settings: Settings,
		private readonly _database: Database,
		private readonly _syncer: Syncer,
		private readonly _syncService: SyncService,
		private readonly _logger: Logger,
		private readonly _connectionStatus: ConnectionStatus
	) {}

	public get syncer(): Syncer {
		return this._syncer;
	}

	public get logger(): Logger {
		return this._logger;
	}

	public get documentCount(): number {
		return this._database.length;
	}

	public static async create(
		fs: FileSystemOperations,
		persistence: PersistenceProvider<
			Partial<{
				settings: Partial<SyncSettings>;
				database: Partial<StoredDatabase>;
			}>
		>,
		fetch: typeof globalThis.fetch = globalThis.fetch
	): Promise<SyncClient> {
		const logger = new Logger();
		logger.info("Starting SyncClient");

		const history = new SyncHistory(logger);

		await initWasm(
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

		const connectionStatus = new ConnectionStatus(settings, logger);
		const syncService = new SyncService(connectionStatus, settings, logger);
		syncService.fetchImplementation = fetch;
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
			logger,
			connectionStatus
		);

		settings.addOnSettingsChangeHandlers((newSettings, oldSettings) => {
			if (
				newSettings.fetchChangesUpdateIntervalMs !==
				oldSettings.fetchChangesUpdateIntervalMs
			) {
				client.setRemoteEventListener(
					newSettings.fetchChangesUpdateIntervalMs
				);
			}

			if (
				newSettings.vaultName !== oldSettings.vaultName ||
				newSettings.token !== oldSettings.token ||
				newSettings.remoteUri !== oldSettings.remoteUri
			) {
				client.reset();
			}
		});

		logger.info("SyncClient initialised");

		return client;
	}

	public async checkConnection(): Promise<CheckConnectionResult> {
		return this._syncService.checkConnection();
	}

	public getHistoryEntries(): HistoryEntry[] {
		return this._history.getEntries();
	}

	public addSyncHistoryUpdateListener(
		listener: (stats: HistoryStats) => void
	): void {
		this._history.addSyncHistoryUpdateListener(listener);
	}

	public async start(): Promise<void> {
		await this._syncer.scheduleSyncForOfflineChanges();

		this.setRemoteEventListener(
			this._settings.getSettings().fetchChangesUpdateIntervalMs
		);
	}

	/// Clear all global state that has been touched by SyncClient.
	public stop(): void {
		this.unsetRemoteEventListener();
	}

	/// Wait for the in-flight operations to finish, reset all tracking,
	/// and the local database but retain the settings.
	/// The SyncClient can be used again after calling this method.
	public async reset(): Promise<void> {
		this.stop();
		this._connectionStatus.reset();
		await this._syncer.reset();
		this._history.reset();
		this._database.reset();
		this._logger.reset();
		void this.start();
	}

	public getSettings(): SyncSettings {
		return this._settings.getSettings();
	}

	public async setSetting<T extends keyof SyncSettings>(
		key: T,
		value: SyncSettings[T]
	): Promise<void> {
		await this._settings.setSetting(key, value);
	}

	public addOnSettingsChangeHandlers(
		handler: (settings: SyncSettings, oldSettings: SyncSettings) => void
	): void {
		this._settings.addOnSettingsChangeHandlers(handler);
	}

	private setRemoteEventListener(intervalMs: number): void {
		if (this.remoteListenerIntervalId !== null) {
			clearInterval(this.remoteListenerIntervalId);
		}

		this.remoteListenerIntervalId = setInterval(
			() => void this._syncer.applyRemoteChangesLocally(),
			intervalMs
		);
	}

	private unsetRemoteEventListener(): void {
		if (this.remoteListenerIntervalId !== null) {
			clearInterval(this.remoteListenerIntervalId);
		}
	}
}
