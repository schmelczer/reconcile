import type { Logger } from "../tracing/logger";
import { LogLevel } from "../tracing/logger";

export interface SyncSettings {
	remoteUri: string;
	token: string;
	vaultName: string;
	fetchChangesUpdateIntervalMs: number;
	syncConcurrency: number;
	isSyncEnabled: boolean;
	maxFileSizeMB: number;
}

const DEFAULT_SETTINGS: SyncSettings = {
	remoteUri: "",
	token: "",
	vaultName: "default",
	fetchChangesUpdateIntervalMs: 1000,
	syncConcurrency: 1,
	isSyncEnabled: false,
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

	public addOnSettingsChangeListener(
		handler: (settings: SyncSettings, oldSettings: SyncSettings) => void
	): void {
		this.onSettingsChangeHandlers.push(handler);
	}

	public async setSetting<T extends keyof SyncSettings>(
		key: T,
		value: SyncSettings[T]
	): Promise<void> {
		this.logger.debug(`Setting '${key}' to '${value}'`);
		await this.setSettings({
			[key]: value
		});
	}

	public async setSettings(value: Partial<SyncSettings>): Promise<void> {
		const oldSettings = this.settings;
		this.settings = {
			...this.settings,
			...value
		};

		this.onSettingsChangeHandlers.forEach((handler) => {
			handler(this.settings, oldSettings);
		});
		await this.save();
	}

	private async save(): Promise<void> {
		await this.saveData(this.settings);
	}
}
