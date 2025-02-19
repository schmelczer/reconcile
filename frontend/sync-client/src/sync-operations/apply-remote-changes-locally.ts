import type { Database } from "../database/database";
import type { SyncService } from "src/services/sync-service";
import { Logger } from "src/tracing/logger";
import type { Syncer } from "./syncer";

let isRunning = false;

export async function applyRemoteChangesLocally({
	database,
	syncService,
	syncer
}: {
	database: Database;
	syncService: SyncService;
	syncer: Syncer;
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
		const remote = await syncService.getAll(database.getLastSeenUpdateId());

		if (remote.latestDocuments.length === 0) {
			Logger.getInstance().debug("No remote changes to apply");
			return;
		}

		Logger.getInstance().info("Applying remote changes locally");

		await Promise.all(
			remote.latestDocuments.map(async (remoteDocument) =>
				syncer.syncRemotelyUpdatedFile(remoteDocument)
			)
		);

		const lastSeenUpdateId = database.getLastSeenUpdateId();
		if (
			lastSeenUpdateId === undefined ||
			remote.lastUpdateId > lastSeenUpdateId
		) {
			await database.setLastSeenUpdateId(remote.lastUpdateId);
		}
	} catch (e) {
		Logger.getInstance().error(
			`Failed to apply remote changes locally: ${e}`
		);
	} finally {
		isRunning = false;
	}
}
