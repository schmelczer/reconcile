import { LogLevel } from "src/tracing/logger";

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

export const DEFAULT_SETTINGS: SyncSettings = {
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
