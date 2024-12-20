import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import type { Database } from "src/database/database";
import type { SyncService } from "src/services/sync-service";
import { hash } from "src/utils/hash";
import { unlockDocument, waitForDocumentLock } from "./locks";
import type { FileOperations } from "src/file-operations/file-operations";
import type { RelativePath } from "src/database/document-metadata";
import type { SyncHistory } from "src/tracing/sync-history.js";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history.js";
import { Logger } from "src/tracing/logger.js";

export async function syncLocallyCreatedFile({
	database,
	syncServer,
	operations,
	history,
	updateTime,
	relativePath,
}: {
	database: Database;
	syncServer: SyncService;
	operations: FileOperations;
	history: SyncHistory;
	updateTime: Date;
	relativePath: RelativePath;
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
		const metadata = database.getDocument(relativePath);
		if (metadata) {
			Logger.getInstance().debug(
				`Document metadata already exists for ${relativePath}, it must have been downloaded from the server`
			);
		}

		const contentBytes = await operations.read(relativePath);
		const contentHash = hash(contentBytes);

		const response = await syncServer.create({
			relativePath,
			contentBytes,
			createdDate: updateTime,
		});

		history.addHistoryEntry({
			status: SyncStatus.SUCCESS,
			source: SyncSource.PUSH,
			relativePath,
			message: `Successfully uploaded locally created file`,
			type: SyncType.CREATE,
		});

		const responseBytes = lib.base64_to_bytes(response.contentBase64);
		const responseHash = hash(responseBytes);

		if (contentHash !== responseHash) {
			await operations.write(relativePath, contentBytes, responseBytes);
			history.addHistoryEntry({
				status: SyncStatus.SUCCESS,
				source: SyncSource.PULL,
				relativePath,
				message: `The file we created locally has already existed remotely, so we have merged them`,
				type: SyncType.UPDATE,
			});
		}

		await database.setDocument({
			documentId: response.documentId,
			relativePath: response.relativePath,
			parentVersionId: response.vaultUpdateId,
			hash: responseHash,
		});
	} catch (e) {
		history.addHistoryEntry({
			status: SyncStatus.ERROR,
			relativePath,
			message: `Failed to reconcile locally created file: ${e}`,
			type: SyncType.CREATE,
		});
		throw e;
	} finally {
		unlockDocument(relativePath);
	}
}
