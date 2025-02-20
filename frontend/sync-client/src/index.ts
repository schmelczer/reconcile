export { Settings, type SyncSettings } from "./persistence/settings";

export { type CheckConnectionResult } from "./services/sync-service";

export { Syncer } from "./sync-operations/syncer";

export {
	SyncHistory,
	SyncType,
	SyncSource,
	SyncStatus,
	type HistoryStats,
	type HistoryEntry
} from "./tracing/sync-history";
export { Logger, LogLevel } from "./tracing/logger";

export { SyncClient } from "./sync-client";
export { type FileOperations } from "./file-operations";
export { type RelativePath } from "./persistence/database";
export type { PersistenceProvider } from "./persistence/persistence";

export {
	isFileTypeMergable,
	mergeText,
	bytesToBase64,
	base64ToBytes,
	merge
} from "sync_lib";
