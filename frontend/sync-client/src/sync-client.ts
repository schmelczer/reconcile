import initWasm from "sync_lib";
import wasmBin from "../../../backend/sync_lib/pkg/sync_lib_bg.wasm";
import type { PersistenceProvider } from "./persistence/persistence";
import type { HistoryEntry, HistoryStats } from "./tracing/sync-history";
import { SyncHistory } from "./tracing/sync-history";
import { Logger } from "./tracing/logger";
import type { RelativePath, StoredDatabase } from "./persistence/database";
import { Database } from "./persistence/database";
import type { SyncSettings } from "./persistence/settings";
import { Settings } from "./persistence/settings";
import { SyncService } from "./services/sync-service";
import { Syncer } from "./sync-operations/syncer";
import type { FileSystemOperations } from "./file-operations/filesystem-operations";
import { FileOperations } from "./file-operations/file-operations";
import { ConnectionStatus } from "./services/connection-status";
import { UnrestrictedSyncer } from "./sync-operations/unrestricted-syncer";
import { rateLimit } from "./utils/rate-limit";
import { v4 as uuidv4 } from "uuid";

export interface NetworkConnectionStatus {
	isSuccessful: boolean;
	serverMessage: string;
	isWebSocketConnected: boolean;
}

export class SyncClient {
	private static readonly MINIMUM_SAVE_INTERVAL_MS = 1000;

	// eslint-disable-next-line @typescript-eslint/max-params
	private constructor(
		private readonly history: SyncHistory,
		private readonly settings: Settings,
		private readonly database: Database,
		private readonly syncer: Syncer,
		private readonly syncService: SyncService,
		private readonly _logger: Logger,
		private readonly connectionStatus: ConnectionStatus
	) {
		this.settings.addOnSettingsChangeListener(
			(newSettings, oldSettings) => {
				if (newSettings.vaultName !== oldSettings.vaultName) {
					void this.reset();
				}
			}
		);
	}

	public get logger(): Logger {
		return this._logger;
	}

	public get documentCount(): number {
		return this.database.length;
	}

	public static async create({
		fs,
		persistence,
		fetch = globalThis.fetch,
		nativeLineEndings = "\n"
	}: {
		fs: FileSystemOperations;
		persistence: PersistenceProvider<
			Partial<{
				settings: Partial<SyncSettings>;
				database: Partial<StoredDatabase>;
			}>
		>;
		fetch?: typeof globalThis.fetch;
		nativeLineEndings?: string;
	}): Promise<SyncClient> {
		const logger = new Logger();
		logger.info("Initialising SyncClient");

		const history = new SyncHistory(logger);

		await initWasm(
			// eslint-disable-next-line
			(wasmBin as any).default // it is loaded as a base64 string by webpack
		);

		let state = (await persistence.load()) ?? {
			settings: undefined,
			database: undefined
		};

		const rateLimitedSave = rateLimit(
			persistence.save,
			SyncClient.MINIMUM_SAVE_INTERVAL_MS
		);

		const database = new Database(
			logger,
			state.database,
			async (data): Promise<void> => {
				state = { ...state, database: data };
				await rateLimitedSave(state);
			}
		);

		const settings = new Settings(
			logger,
			state.settings,
			async (data): Promise<void> => {
				state = { ...state, settings: data };
				await rateLimitedSave(state);
			}
		);
		const deviceId = uuidv4();

		const connectionStatus = new ConnectionStatus(settings, logger);
		const syncService = new SyncService(
			deviceId,
			connectionStatus,
			settings,
			logger
		);
		syncService.fetchImplementation = fetch;
		const fileOperations = new FileOperations(
			logger,
			database,
			fs,
			nativeLineEndings
		);
		const unrestrictedSyncer = new UnrestrictedSyncer(
			logger,
			database,
			settings,
			syncService,
			fileOperations,
			history
		);
		const syncer = new Syncer(
			deviceId,
			logger,
			database,
			settings,
			syncService,
			fileOperations,
			unrestrictedSyncer
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

		logger.info("SyncClient initialised");

		return client;
	}

	public async checkConnection(): Promise<NetworkConnectionStatus> {
		const server = await this.syncService.checkConnection();
		return {
			isSuccessful: server.isSuccessful,
			serverMessage: server.message,
			isWebSocketConnected: this.syncer.isWebSocketConnected
		};
	}

	public getHistoryEntries(): readonly HistoryEntry[] {
		return this.history.entries;
	}

	public addSyncHistoryUpdateListener(
		listener: (stats: HistoryStats) => void
	): void {
		this.history.addSyncHistoryUpdateListener(listener);
	}

	public async start(): Promise<void> {
		await this.syncer.scheduleSyncForOfflineChanges();
	}

	public stop(): void {
		this.syncer.stop();
	}

	public async waitAndStop(): Promise<void> {
		this.stop();
		await this.syncer.waitUntilFinished();
	}

	/// Wait for the in-flight operations to finish, reset all tracking,
	/// and the local database but retain the settings.
	/// The SyncClient can be used again after calling this method.
	public async reset(): Promise<void> {
		this.stop();
		this.connectionStatus.startReset();
		await this.syncer.reset();
		this.history.reset();
		this.database.reset();
		this._logger.reset();
		this.connectionStatus.finishReset();
		void this.start();
	}

	public getSettings(): SyncSettings {
		return this.settings.getSettings();
	}

	public async setSetting<T extends keyof SyncSettings>(
		key: T,
		value: SyncSettings[T]
	): Promise<void> {
		await this.settings.setSetting(key, value);
	}

	public async setSettings(value: Partial<SyncSettings>): Promise<void> {
		await this.settings.setSettings(value);
	}

	public addOnSettingsChangeListener(
		handler: (settings: SyncSettings, oldSettings: SyncSettings) => void
	): void {
		this.settings.addOnSettingsChangeListener(handler);
	}

	public addRemainingSyncOperationsListener(
		listener: (remainingOperations: number) => void
	): void {
		this.syncer.addRemainingOperationsListener(listener);
	}

	public addWebSocketStatusChangeListener(listener: () => void): void {
		this.syncer.addWebSocketStatusChangeListener(listener);
	}

	public async syncLocallyCreatedFile(
		relativePath: RelativePath
	): Promise<void> {
		return this.syncer.syncLocallyCreatedFile(relativePath);
	}

	public async syncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		return this.syncer.syncLocallyDeletedFile(relativePath);
	}

	public async syncLocallyUpdatedFile({
		oldPath,
		relativePath
	}: {
		oldPath?: RelativePath;
		relativePath: RelativePath;
	}): Promise<void> {
		return this.syncer.syncLocallyUpdatedFile({
			oldPath,
			relativePath
		});
	}
}
