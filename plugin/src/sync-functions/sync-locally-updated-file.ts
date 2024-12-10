import { TFile } from "obsidian";
import { Database } from "src/database/database";
import { SyncServer } from "src/services/sync_service";

export async function syncLocallyUpdatedFile(
	database: Database,
	syncServer: SyncServer,
	file: TFile
) {
	const metadata = database.getDocument(file.path);
	if (!metadata) {
		throw `Document metadata not found for ${file.path}`;
	}

	const response = await syncServer.update({
		documentId: metadata.documentId,
		parentVersionId: metadata.parentVersionId,
		relativePath: file.path,
		content: await file.vault.readBinary(file),
		createdDate: new Date(file.stat.ctime),
	});

	if (file.path !== response.relativePath) {
		file.vault.rename(file, response.relativePath);
	}

	if ((await file.vault.read(file)) !== response.contentBase64) {
		// todo - reconcile
	}

	await database.setDocument({
		relativePath: file.path,
		documentId: response.documentId,
		parentVersionId: response.versionId,
	});
}
