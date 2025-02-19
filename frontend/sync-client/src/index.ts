export { applyRemoteChangesLocally } from "./sync-operations/apply-remote-changes-locally";

export {
	Database,
	type RelativePath,
	type DocumentId,
	type VaultUpdateId,
	type DocumentMetadata
} from "./persistence/database";

export { Settings, type SyncSettings } from "./persistence/settings";

export {
	SyncService,
	type CheckConnectionResult
} from "./services/sync-service";

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

export { type FileOperations } from "./file-operations";

import init from "sync_lib";
import wasmBin from "sync_lib/sync_lib_bg.wasm";

export {
	isFileTypeMergable,
	mergeText,
	bytesToBase64,
	base64ToBytes,
	merge,
	isBinary
} from "sync_lib";

export const initialize = async (): Promise<void> => {
	await init(
		// eslint-disable-next-line
		(wasmBin as any).default // it is loaded as a base64 string by webpack
	);
};
