import type { Database } from "src/database/database";
import type { FileOperations } from "src/file-operations/file-operations";
import type { SyncService } from "src/services/sync-service";
import { syncRemotelyUpdatedFile } from "./sync-remotely-updated-file";
import { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";

let isRunning = false;

export async function applyRemoteChangesLocally({
	database,
	syncServer,
	operations,
	history,
}: {
	database: Database;
	syncServer: SyncService;
	operations: FileOperations;
	history: SyncHistory;
}): Promise<void> {
	if (!database.getSettings().isSyncEnabled) {
		Logger.getInstance().debug(
			`Syncing is disabled, not fetching remote changes`
		);
		return;
	} else if (isRunning) {
		Logger.getInstance().debug(
			"Applying remote changes locally is already in progress, skipping invocation"
		);
		return;
	}

	isRunning = true;

	try {
		const remote = await syncServer.getAll(database.getLastSeenUpdateId());

		if (remote.latestDocuments.length === 0) {
			Logger.getInstance().debug("No remote changes to apply");
			return;
		}

		Logger.getInstance().info("Applying remote changes locally");

		await Promise.all(
			remote.latestDocuments.map(async (remoteDocument) =>
				syncRemotelyUpdatedFile({
					database,
					syncServer,
					history,
					operations,
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
