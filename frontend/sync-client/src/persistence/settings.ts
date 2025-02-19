import { Logger, LogLevel } from "src/tracing/logger";

export interface SyncSettings {
	remoteUri: string;
	token: string;
	vaultName: string;
	fetchChangesUpdateIntervalMs: number;
	syncConcurrency: number;
	isSyncEnabled: boolean;
	displayNoopSyncEvents: boolean;
	minimumLogLevel: LogLevel;
	maxFileSizeMB: number;
}

const DEFAULT_SETTINGS: SyncSettings = {
	remoteUri: "",
	token: "",
	vaultName: "default",
	fetchChangesUpdateIntervalMs: 1000,
	syncConcurrency: 1,
	isSyncEnabled: false,
	displayNoopSyncEvents: false,
	minimumLogLevel: LogLevel.INFO,
	maxFileSizeMB: 10
};

export class Settings {
	private settings: SyncSettings;

	private readonly onSettingsChangeHandlers: ((
		newSettings: SyncSettings,
		oldSettings: SyncSettings
	) => void)[] = [];

	public constructor(
		initialState: Partial<SyncSettings> | undefined,
		private readonly saveData: (data: unknown) => Promise<void>
	) {
		this.settings = {
			...DEFAULT_SETTINGS,
			...(initialState ?? {})
		};

		Logger.getInstance().debug(
			`Loaded settings: ${JSON.stringify(this.settings, null, 2)}`
		);
	}

	public getSettings(): SyncSettings {
		return this.settings;
	}

	public async setSettings(value: SyncSettings): Promise<void> {
		const oldSettings = this.settings;
		this.settings = value;
		this.onSettingsChangeHandlers.forEach((handler) => {
			handler(value, oldSettings);
		});
		await this.save();
	}

	public addOnSettingsChangeHandlers(
		handler: (settings: SyncSettings, oldSettings: SyncSettings) => void
	): void {
		this.onSettingsChangeHandlers.push(handler);
	}

	public async setSetting<T extends keyof SyncSettings>(
		key: T,
		value: SyncSettings[T]
	): Promise<void> {
		const newSettings = { ...this.settings, [key]: value };
		Logger.getInstance().debug(
			`Setting ${key} to ${value}, new settings: ${JSON.stringify(
				newSettings,
				null,
				2
			)}`
		);
		await this.setSettings(newSettings);
	}

	private async save(): Promise<void> {
		await this.saveData(this.settings);
	}
}
