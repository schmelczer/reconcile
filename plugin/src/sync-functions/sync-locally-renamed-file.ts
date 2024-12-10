import { TFile } from "obsidian";
import { Database } from "src/database/database";
import { SyncServer } from "src/services/sync_service";

export async function syncLocallyRenamedFile(
	database: Database,
	syncServer: SyncServer,
	file: TFile,
	oldPath: string
) {
	const metadata = database.getDocument(oldPath);
	if (!metadata) {
		throw `Document metadata not found for ${oldPath}`;
	}

	const response = await syncServer.update({
		documentId: metadata.documentId,
		parentVersionId: metadata.parentVersionId,
		relativePath: file.path,
		content: await file.vault.readBinary(file),
		createdDate: new Date(file.stat.ctime),
	});

	await database.moveDocument({
		oldRelativePath: oldPath,
		relativePath: file.path,
		documentId: response.documentId,
		parentVersionId: response.versionId,
	});
}
