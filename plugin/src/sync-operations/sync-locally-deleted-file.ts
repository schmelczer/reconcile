import { Database } from "src/database/database";
import { RelativePath } from "src/database/document-metadata";
import { Logger } from "src/logger";
import { SyncServer } from "src/services/sync_service";
import { unlockDocument, waitForDocumentLock } from "./locks";

export async function syncLocallyDeletedFile(
	database: Database,
	syncServer: SyncServer,
	relativePath: RelativePath
): Promise<void> {
	await waitForDocumentLock(relativePath);

	try {
		const metadata = database.getDocument(relativePath);
		if (!metadata) {
			Logger.getInstance().warn(
				`Document metadata not found for ${relativePath}`
			);
			return;
		}

		await syncServer.delete({
			documentId: metadata.documentId,
			relativePath,
			// We got the event now, so it must have been deleted just now
			createdDate: new Date(),
		});

		await database.removeDocument(relativePath);
	} catch (e) {
		Logger.getInstance().error(
			`Failed to sync locally deleted file ${relativePath}: ${e}`
		);
	} finally {
		unlockDocument(relativePath);
	}
}
