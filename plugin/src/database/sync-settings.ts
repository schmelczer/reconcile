export interface SyncSettings {
	remoteUri: string;
	token: string;
	fetchChangesUpdateIntervalMs: number;
	isSyncEnabled: boolean;
}

export const DEFAULT_SETTINGS: SyncSettings = {
	remoteUri: "",
	token: "",
	fetchChangesUpdateIntervalMs: 1000,
	isSyncEnabled: true,
};
