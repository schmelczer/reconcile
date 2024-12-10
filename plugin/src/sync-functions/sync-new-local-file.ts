import { TFile } from "obsidian";
import { Database } from "src/database/database";
import { SyncServer } from "src/services/sync_service";

export async function syncNewLocalFile(
	database: Database,
	syncServer: SyncServer,
	file: TFile
) {
	const response = await syncServer.create({
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
		relativePath: response.relativePath,
		documentId: response.documentId,
		parentVersionId: response.versionId,
	});
}
