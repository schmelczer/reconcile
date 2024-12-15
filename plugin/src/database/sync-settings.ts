export interface SyncSettings {
	remoteUri: string;
	token: string;
	fetchChangesUpdateInterval: number;
	isSyncEnabled: boolean;
}

export const DEFAULT_SETTINGS: SyncSettings = {
	remoteUri: "",
	token: "",
	fetchChangesUpdateInterval: 1,
	isSyncEnabled: true,
};
