import { Database } from "src/database/database";
import { RelativePath } from "src/database/document-metadata";
import { Logger } from "src/logger";
import { SyncServer } from "src/services/sync_service";

export async function syncLocallyDeletedFile(
	database: Database,
	syncServer: SyncServer,
	relativePath: RelativePath
) {
	const metadata = database.getDocument(relativePath);
	if (!metadata) {
		Logger.getInstance().warn(
			`Document metadata not found for ${relativePath}`
		);
	}

	await syncServer.delete({
		relativePath,
		// We got the event now, so it must have been deleted just now
		createdDate: new Date(),
	});

	if (metadata) {
		await database.removeDocument(relativePath);
	}
}
