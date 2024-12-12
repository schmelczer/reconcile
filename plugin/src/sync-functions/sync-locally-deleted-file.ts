import { Database } from "src/database/database";
import { RelativePath } from "src/database/document-metadata";
import { SyncServer } from "src/services/sync_service";

export async function syncLocallyDeletedFile(
	database: Database,
	syncServer: SyncServer,
	path: RelativePath
) {
	const metadata = database.getDocument(path);
	if (!metadata) {
		throw `Document metadata not found for ${path}`;
	}

	await syncServer.delete({
		documentId: metadata.documentId,
		// We got the event now, so it must have been deleted just now
		createdDate: new Date(),
	});

	await database.removeDocument(path);
}
