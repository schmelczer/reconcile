import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { TFile } from "obsidian";
import { Database } from "src/database/database";
import { Logger } from "src/logger";
import { SyncServer } from "src/services/sync_service";
import { hash, isEqualBytes } from "src/utils";

export async function syncLocallyUpdatedFile({
	database,
	syncServer,
	file,
	oldPath,
}: {
	database: Database;
	syncServer: SyncServer;
	file: TFile;
	oldPath?: string;
}) {
	const contentBytes = new Uint8Array(await file.vault.readBinary(file));
	const contentHash = hash(contentBytes);

	const metadata = database.getDocument(oldPath || file.path);
	if (!metadata) {
		Logger.getInstance().info(
			`Document metadata not found for ${
				oldPath || file.path
			}, it must be new`
		);
	} else if (metadata.hash === contentHash) {
		Logger.getInstance().info(
			`Document hash matches, no need to sync ${file.path}`
		);
		return;
	}

	const response = await syncServer.put({
		parentVersionId: metadata?.parentVersionId,
		relativePath: file.path,
		contentBytes,
		createdDate: new Date(file.stat.ctime),
	});

	const localDbUpdatePromise = database.moveDocument({
		oldRelativePath: oldPath || file.path,
		relativePath: file.path,
		parentVersionId: response.versionId,
		hash: contentHash,
	});

	if (file.path !== response.relativePath) {
		await file.vault.rename(file, response.relativePath);
	}

	const newContentBytes = new Uint8Array(await file.vault.readBinary(file));
	const responseBytes = lib.base64_to_bytes(response.contentBase64);

	if (!isEqualBytes(contentBytes, newContentBytes)) {
		Logger.getInstance().info(
			`Content changed since sending original update request for ${file.path}`
		);

		const result = lib.merge(contentBytes, newContentBytes, responseBytes);

		await file.vault.modifyBinary(file, result);
	} else {
		await file.vault.modifyBinary(file, responseBytes);
	}

	await localDbUpdatePromise;
}
