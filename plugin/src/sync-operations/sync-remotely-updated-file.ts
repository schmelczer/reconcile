import type { Database } from "src/database/database";
import { unlockDocument, waitForDocumentLock } from "./locks";
import type { SyncService } from "src/services/sync-service";
import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { hash } from "src/utils/hash";
import type { components } from "src/services/types";
import type { FileOperations } from "src/file-operations/file-operations";
import { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history";

export async function syncRemotelyUpdatedFile({
	database,
	syncServer,
	operations,
	history,
	remoteVersion,
}: {
	database: Database;
	syncServer: SyncService;
	operations: FileOperations;
	history: SyncHistory;
	remoteVersion: components["schemas"]["DocumentVersionWithoutContent"];
}): Promise<void> {
	Logger.getInstance().debug(
		`Syncing remotely updated file ${remoteVersion.relativePath}`
	);

	const content = (
		await syncServer.get({
			documentId: remoteVersion.documentId,
		})
	).contentBase64;
	const contentBytes = lib.base64_to_bytes(content);
	const contentHash = hash(contentBytes);

	await waitForDocumentLock(remoteVersion.relativePath);

	try {
		const currentVersion = database.getDocumentByDocumentId(
			remoteVersion.documentId
		);

		if (!currentVersion) {
			if (remoteVersion.isDeleted) {
				history.addHistoryEntry({
					status: SyncStatus.NO_OP,
					source: SyncSource.PULL,
					relativePath: remoteVersion.relativePath,
					message: `Remotely deleted file hasn't been synced yet, so there's no need to delete it locally`,
					type: SyncType.DELETE,
				});
				return;
			}

			await operations.create(remoteVersion.relativePath, contentBytes);
			await database.setDocument({
				documentId: remoteVersion.documentId,
				relativePath: remoteVersion.relativePath,
				parentVersionId: remoteVersion.vaultUpdateId,
				hash: contentHash,
			});
			history.addHistoryEntry({
				status: SyncStatus.SUCCESS,
				source: SyncSource.PULL,
				relativePath: remoteVersion.relativePath,
				message: `Successfully downloaded remote file which hasn't existed locally`,
				type: SyncType.CREATE,
			});
			return;
		}

		const [relativePath, metadata] = currentVersion;
		if (relativePath !== remoteVersion.relativePath) {
			await waitForDocumentLock(relativePath);
		}
		try {
			if (remoteVersion.isDeleted) {
				await operations.remove(relativePath);
				await database.removeDocument(relativePath);

				history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PULL,
					relativePath: remoteVersion.relativePath,
					message: `Successfully deleted remotely deleted file locally`,
					type: SyncType.DELETE,
				});
			} else {
				const currentContent = await operations.read(relativePath);
				const currentHash = hash(currentContent);

				if (currentHash !== metadata.hash) {
					Logger.getInstance().info(
						`Document ${relativePath} has been updated both remotely and locally, skipping until the event is processed`
					);
				} else if (contentHash !== metadata.hash) {
					if (relativePath !== remoteVersion.relativePath) {
						await operations.move(
							relativePath,
							remoteVersion.relativePath
						);
					}

					await operations.write(
						remoteVersion.relativePath,
						currentContent,
						contentBytes
					);
					await database.moveDocument({
						documentId: remoteVersion.documentId,
						oldRelativePath: relativePath,
						relativePath: remoteVersion.relativePath,
						parentVersionId: remoteVersion.vaultUpdateId,
						hash: contentHash,
					});

					history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						source: SyncSource.PULL,
						relativePath: remoteVersion.relativePath,
						message: `Successfully updated remotely updated file locally`,
						type: SyncType.UPDATE,
					});
				}
				{
					Logger.getInstance().debug(
						`Document ${relativePath} is already up to date`
					);
				}
			}
		} finally {
			if (relativePath !== remoteVersion.relativePath) {
				unlockDocument(relativePath);
			}
		}
	} catch (e) {
		history.addHistoryEntry({
			status: SyncStatus.ERROR,
			source: SyncSource.PULL,
			relativePath: remoteVersion.relativePath,
			message: `Failed to reconcile remotely updated file: ${e}`,
		});
		throw e;
	} finally {
		unlockDocument(remoteVersion.relativePath);
	}
}
