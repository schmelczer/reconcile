import SyncPlugin from "src/plugin";

export interface SyncSettings {
	remoteUri: string;
	token: string;
	fullScanIntervalInSeconds: number;
	fullScanEnabled: boolean;
}

export const DEFAULT_SETTINGS: SyncSettings = {
	remoteUri: "",
	token: "",
	fullScanIntervalInSeconds: 60,
	fullScanEnabled: true,
};

export class SettingsContainer {
	private _settings: SyncSettings;

	private onChangeHandlers: Array<(settings: SyncSettings) => void> = [];

	public constructor(
		private plugin: SyncPlugin,
		private loadedSettings: any
	) {
		this._settings = Object.assign({}, DEFAULT_SETTINGS, loadedSettings);
	}

	public onChange(handler: (settings: SyncSettings) => void) {
		this.onChangeHandlers.push(handler);
	}

	public getSettings(): SyncSettings {
		return this._settings;
	}

	public async setSettings(value: SyncSettings): Promise<void> {
		this._settings = value;
		await this.plugin.saveData(value);
		this.onChangeHandlers.forEach((handler) => handler(value));
	}

	public async setSetting<T extends keyof SyncSettings>(
		key: T,
		value: SyncSettings[T]
	): Promise<void> {
		this._settings[key] = value;
		await this.plugin.saveData(value);
		this.onChangeHandlers.forEach((handler) => handler(this._settings));
	}
}
