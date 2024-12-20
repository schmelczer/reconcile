import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import type { Database } from "src/database/database";
import type { SyncService } from "src/services/sync-service";
import { hash } from "src/utils/hash";
import { unlockDocument, waitForDocumentLock } from "./locks";
import type { FileOperations } from "src/file-operations/file-operations";
import type { RelativePath } from "src/database/document-metadata";
import { Logger } from "src/tracing/logger.js";
import type { SyncHistory } from "src/tracing/sync-history.js";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history.js";

/// This can be used when updating a file's content and/or path.
export async function syncLocallyUpdatedFile({
	database,
	syncServer,
	operations,
	history,
	updateTime,
	relativePath,
	oldPath,
}: {
	database: Database;
	syncServer: SyncService;
	operations: FileOperations;
	history: SyncHistory;
	updateTime: Date;
	relativePath: RelativePath;
	oldPath?: RelativePath;
}): Promise<void> {
	if (!database.getSettings().isSyncEnabled) {
		Logger.getInstance().info(
			`Syncing is disabled, not syncing ${relativePath}`
		);
		return;
	}
	Logger.getInstance().debug(`Syncing ${relativePath}`);

	await waitForDocumentLock(relativePath);

	try {
		const metadata = database.getDocument(oldPath ?? relativePath);
		if (!metadata) {
			throw new Error(
				`Document metadata not found for ${relativePath}. Consider resetting the plugin's sync history.`
			);
		}

		const contentBytes = await operations.read(relativePath);
		const contentHash = hash(contentBytes);

		if (metadata.hash === contentHash && oldPath !== undefined) {
			history.addHistoryEntry({
				status: SyncStatus.NO_OP,
				relativePath,
				message: `File hash matches with last synced version, no need to sync`,
				type: SyncType.UPDATE,
			});
			return;
		}

		const response = await syncServer.put({
			documentId: metadata.documentId,
			parentVersionId: metadata.parentVersionId,
			relativePath,
			contentBytes,
			createdDate: updateTime,
		});

		history.addHistoryEntry({
			status: SyncStatus.SUCCESS,
			source: SyncSource.PUSH,
			relativePath,
			message: `Successfully uploaded locally updated file to the remote server`,
			type: SyncType.UPDATE,
		});

		if (response.isDeleted) {
			await operations.remove(oldPath ?? relativePath);
			await database.removeDocument(oldPath ?? relativePath);

			history.addHistoryEntry({
				status: SyncStatus.SUCCESS,
				source: SyncSource.PULL,
				relativePath,
				message:
					"The file we tried to update had been deleted remotely, therefore, we have deleted it locally",
				type: SyncType.DELETE,
			});

			return;
		}

		const responseBytes = lib.base64_to_bytes(response.contentBase64);
		const responseHash = hash(responseBytes);

		if (response.relativePath != relativePath) {
			await waitForDocumentLock(response.relativePath);

			try {
				await operations.move(
					oldPath ?? relativePath,
					response.relativePath
				);
				await operations.write(
					response.relativePath,
					contentBytes,
					responseBytes
				);
				history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PULL,
					relativePath,
					message:
						"The file we updated had been moved remotely, therefore, we have moved it locally as well",
					type: SyncType.UPDATE,
				});
			} finally {
				unlockDocument(response.relativePath);
			}
		} else if (contentHash !== responseHash) {
			await operations.write(relativePath, contentBytes, responseBytes);
		}

		await database.moveDocument({
			documentId: metadata.documentId,
			oldRelativePath: oldPath ?? relativePath,
			relativePath: response.relativePath,
			parentVersionId: response.vaultUpdateId,
			hash: responseHash,
		});
	} catch (e) {
		history.addHistoryEntry({
			status: SyncStatus.ERROR,
			relativePath,
			message: `Failed to reconcile locally updated file: ${e}`,
			type: SyncType.UPDATE,
		});
		throw e;
	} finally {
		unlockDocument(relativePath);
	}
}
