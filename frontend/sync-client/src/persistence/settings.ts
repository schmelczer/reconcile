import type { Logger } from "../tracing/logger";
import { LogLevel } from "../tracing/logger";

export interface SyncSettings {
	remoteUri: string;
	token: string;
	vaultName: string;
	fetchChangesUpdateIntervalMs: number;
	syncConcurrency: number;
	isSyncEnabled: boolean;
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
		private readonly logger: Logger,
		initialState: Partial<SyncSettings> | undefined,
		private readonly saveData: (data: SyncSettings) => Promise<void>
	) {
		this.settings = {
			...DEFAULT_SETTINGS,
			...(initialState ?? {})
		};

		this.logger.debug(
			`Loaded settings: ${JSON.stringify(this.settings, null, 2)}`
		);
	}

	public getSettings(): SyncSettings {
		return this.settings;
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
		this.logger.debug(`Setting '${key}' to '${value}'`);
		await this.setSettings(newSettings);
	}

	private async setSettings(value: SyncSettings): Promise<void> {
		const oldSettings = this.settings;
		this.settings = value;
		this.onSettingsChangeHandlers.forEach((handler) => {
			handler(value, oldSettings);
		});
		await this.save();
	}

	private async save(): Promise<void> {
		await this.saveData(this.settings);
	}
}
