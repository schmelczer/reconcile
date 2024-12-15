import { Vault } from "obsidian";
import { Database } from "src/database/database";
import { unlockDocument, waitForDocumentLock } from "./locks";
import { SyncServer } from "src/services/sync_service";
import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { hash } from "src/utils/hash";
import { Logger } from "src/logger";
import { components } from "src/services/types";
import { FileOperations } from "src/file-operations/file-operations";

export async function syncRemotelyUpdatedFile({
	database,
	syncServer,
	operations,
	remoteVersion,
}: {
	database: Database;
	syncServer: SyncServer;
	operations: FileOperations;
	remoteVersion: components["schemas"]["DocumentVersionWithoutContent"];
}): Promise<void> {
	Logger.getInstance().info(
		`Syncing remotely updated file ${remoteVersion.relativePath}`
	);
	const content = (
		await syncServer.get({
			documentId: remoteVersion.documentId,
		})
	).contentBase64;

	const currentVersion = database.getDocumentByDocumentId(
		remoteVersion.documentId
	);

	if (!currentVersion) {
		if (remoteVersion.isDeleted) {
			return;
		}

		Logger.getInstance().info(
			`Document metadata not found for ${remoteVersion.relativePath}, it must be new`
		);

		await waitForDocumentLock(remoteVersion.relativePath);
		try {
			const contentBytes = lib.base64_to_bytes(content);
			operations.create(remoteVersion.relativePath, contentBytes);
			await database.setDocument({
				documentId: remoteVersion.documentId,
				relativePath: remoteVersion.relativePath,
				parentVersionId: remoteVersion.vaultUpdateId,
				hash: hash(contentBytes),
			});
		} finally {
			unlockDocument(remoteVersion.relativePath);
		}
		return;
	}

	const [relativePath, metadata] = currentVersion;
	await waitForDocumentLock(relativePath);

	try {
		if (remoteVersion.isDeleted) {
			Logger.getInstance().info(
				`Document ${relativePath} has been deleted remotely`
			);
			await operations.remove(relativePath);

			if (metadata) {
				await database.removeDocument(relativePath);
			}
		} else {
			const currentContent = await operations.read(relativePath);
			const currentHash = hash(currentContent);
			if (currentHash !== metadata.hash) {
				Logger.getInstance().info(
					`Document ${relativePath} has been updated both remotely and locally, skipping`
				);
				return;
			} else {
				if (relativePath !== remoteVersion.relativePath) {
					await operations.move(
						relativePath,
						remoteVersion.relativePath
					);
				}

				const contentBytes = lib.base64_to_bytes(content);
				await operations.write(
					remoteVersion.relativePath,
					currentContent,
					contentBytes
				);
				await database.moveDocument({
					documentId: remoteVersion.documentId,
					oldRelativePath: relativePath,
					relativePath: remoteVersion.relativePath,
					parentVersionId: remoteVersion.vaultUpdateId,
					hash: metadata.hash,
				});
			}
		}
	} catch (e) {
		Logger.getInstance().error(
			`Failed to sync remotely updated file ${remoteVersion.relativePath}: ${e}`
		);
	} finally {
		unlockDocument(relativePath);
	}
}
