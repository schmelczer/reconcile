import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { Database } from "src/database/database";
import { Logger } from "src/logger";
import { SyncServer } from "src/services/sync_service";
import { hash } from "src/utils/hash";
import { unlockDocument, waitForDocumentLock } from "./locks.js";
import { FileOperations } from "src/file-operations/file-operations.js";
import { RelativePath } from "src/database/document-metadata.js";

/// This can be used when updating a files content and/or path.
export async function syncLocallyUpdatedFile({
	database,
	syncServer,
	operations,
	updateTime,
	filePath,
	oldPath,
}: {
	database: Database;
	syncServer: SyncServer;
	operations: FileOperations;
	updateTime: Date;
	filePath: RelativePath;
	oldPath?: RelativePath;
}): Promise<void> {
	await waitForDocumentLock(filePath);

	try {
		const metadata = database.getDocument(oldPath || filePath);
		if (!metadata) {
			throw new Error(`Document metadata not found for ${filePath}`);
		}

		const contentBytes = await operations.read(filePath);
		const contentHash = hash(contentBytes);

		if (metadata.hash === contentHash && !oldPath) {
			Logger.getInstance().info(
				`Document hash matches, no need to sync ${filePath}`
			);
			return;
		}

		const response = await syncServer.put({
			documentId: metadata.documentId,
			parentVersionId: metadata.parentVersionId,
			relativePath: filePath,
			contentBytes,
			createdDate: updateTime,
		});

		if (response.isDeleted) {
			await operations.remove(oldPath || filePath);

			if (metadata) {
				await database.removeDocument(oldPath || filePath);
			}

			return;
		}

		const responseBytes = lib.base64_to_bytes(response.contentBase64);

		if (response.relativePath != filePath) {
			await waitForDocumentLock(response.relativePath);
			try {
				await operations.move(
					oldPath || filePath,
					response.relativePath
				);
				await operations.write(
					response.relativePath,
					contentBytes,
					responseBytes
				);
			} finally {
				unlockDocument(response.relativePath);
			}
		} else {
			await operations.write(filePath, contentBytes, responseBytes);
		}

		await database.moveDocument({
			documentId: metadata.documentId,
			oldRelativePath: oldPath || filePath,
			relativePath: response.relativePath,
			parentVersionId: response.vaultUpdateId,
			hash: contentHash,
		});
	} catch (e) {
		Logger.getInstance().error(
			`Failed to sync locally updated file ${filePath}: ${e}`
		);
	} finally {
		unlockDocument(filePath);
	}
}
