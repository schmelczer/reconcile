import { Database } from "src/database/database";
import { FileOperations } from "src/file-operations/file-operations";
import { Logger } from "src/logger";
import { SyncService } from "src/services/sync-service";
import { syncRemotelyUpdatedFile } from "./sync-remotely-updated-file";

let isRunning = false;

export async function applyRemoteChangesLocally(
	database: Database,
	syncServer: SyncService,
	operations: FileOperations
) {
	if (isRunning) {
		Logger.getInstance().info("Pull sync already in progress, skipping");
		return;
	} else {
		Logger.getInstance().info("Starting pull sync");
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
