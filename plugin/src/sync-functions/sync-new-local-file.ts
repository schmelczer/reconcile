import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { TFile } from "obsidian";
import { Database } from "src/database/database";
import { Logger } from "src/logger.js";
import { SyncServer } from "src/services/sync_service";
import { hash } from "src/utils";

export async function syncNewLocalFile(
	database: Database,
	syncServer: SyncServer,
	file: TFile
) {
	const contentBytes = new Uint8Array(await file.vault.readBinary(file));
	const responsePromise = syncServer.create({
		relativePath: file.path,
		contentBytes,
		createdDate: new Date(file.stat.ctime),
	});

	const contentHash = hash(contentBytes);
	const response = await responsePromise;

	const localDbUpdatePromise = database.setDocument({
		relativePath: response.relativePath,
		documentId: response.documentId,
		parentVersionId: response.versionId,
		hash: contentHash,
	});

	if (file.path !== response.relativePath) {
		await file.vault.rename(file, response.relativePath);
	}

	const newContentBytes = new Uint8Array(await file.vault.readBinary(file));
	const responseBytes = lib.base64_to_bytes(response.contentBase64);

	if (contentBytes !== newContentBytes) {
		Logger.getInstance().info(
			`Content changed since sending original create request for ${file.path}`
		);

		const result = lib.merge(contentBytes, newContentBytes, responseBytes);

		await file.vault.modifyBinary(file, result);
	}

	await localDbUpdatePromise;
}
