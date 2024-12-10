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
