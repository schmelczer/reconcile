import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { Database } from "src/database/database";
import { Logger } from "src/logger";
import { SyncService } from "src/services/sync_service";
import { hash } from "src/utils/hash";
import { unlockDocument, waitForDocumentLock } from "./locks";
import { FileOperations } from "src/file-operations/file-operations";
import { RelativePath } from "src/database/document-metadata";

/// This can be used when updating a files content and/or path.
export async function syncLocallyCreatedFile({
	database,
	syncServer,
	operations,
	updateTime,
	filePath,
}: {
	database: Database;
	syncServer: SyncService;
	operations: FileOperations;
	updateTime: Date;
	filePath: RelativePath;
}): Promise<void> {
	await waitForDocumentLock(filePath);

	try {
		const metadata = database.getDocument(filePath);
		if (metadata) {
			throw new Error(
				`Document metadata found for ${filePath}, this is unexpected`
			);
		}

		const contentBytes = await operations.read(filePath);

		const response = await syncServer.create({
			relativePath: filePath,
			contentBytes,
			createdDate: updateTime,
		});

		const responseBytes = lib.base64_to_bytes(response.contentBase64);
		await operations.write(filePath, contentBytes, responseBytes);
		await database.setDocument({
			documentId: response.documentId,
			relativePath: response.relativePath,
			parentVersionId: response.vaultUpdateId,
			hash: hash(responseBytes),
		});
	} catch (e) {
		Logger.getInstance().error(
			`Failed to sync locally updated file ${filePath}: ${e}`
		);
	} finally {
		unlockDocument(filePath);
	}
}
