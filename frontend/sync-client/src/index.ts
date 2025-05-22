export {
	SyncType,
	SyncStatus,
	type HistoryStats,
	type HistoryEntry
} from "./tracing/sync-history";
export { Logger, LogLevel, LogLine } from "./tracing/logger";
export { type SyncSettings, DEFAULT_SETTINGS } from "./persistence/settings";
export { rateLimit } from "./utils/rate-limit";
export type { RelativePath, StoredDatabase } from "./persistence/database";
export type {
	FileSystemOperations,
	TextWithCursors,
	Cursor
} from "./file-operations/filesystem-operations";
export type { PersistenceProvider } from "./persistence/persistence";

export type { NetworkConnectionStatus } from "./types/network-connection-status";
export { DocumentUpdateStatus } from "./types/document-update-status";
export { SyncClient } from "./sync-client";
