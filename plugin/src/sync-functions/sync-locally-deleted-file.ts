import { TFile } from "obsidian";
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
		createdDate: new Date(), // We got the event now, so it must have been deleted now
	});

	await database.removeDocument(path);
}
