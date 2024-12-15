import { Vault } from "obsidian";
import { Database } from "./database/database";
import { SyncServer } from "./services/sync_service";
import { syncRemotelyUpdatedFile } from "./sync-operations/sync-remotely-updated-file";
import { Logger } from "./logger";
import { FileOperations } from "./file-operations/file-operations";

let isRunning = false;

export async function applyRemoteChangesLocally(
	database: Database,
	syncServer: SyncServer,
	operations: FileOperations
) {
	if (isRunning) {
		Logger.getInstance().info("Sync already in progress, skipping");
		return;
	}

	isRunning = true;
	try {
		if (!database.getSettings().isSyncEnabled) {
			return;
		}

		const remote = await syncServer.getAll(database.getLastSeenUpdateId());

		if (remote.latestDocuments.length === 0) {
			Logger.getInstance().debug("No remote changes to apply");
			return;
		}

		Logger.getInstance().info("Applying remote changes locally");

		await Promise.all(
			remote.latestDocuments.map((remoteDocument) =>
				syncRemotelyUpdatedFile({
					database,
					syncServer,
					operations: operations,
					remoteVersion: remoteDocument,
				})
			)
		);

		await database.setLastSeenUpdateId(remote.lastUpdateId);
	} catch (e) {
		Logger.getInstance().error(
			`Failed to apply remote changes locally: ${e}`
		);
	} finally {
		isRunning = false;
	}
}
