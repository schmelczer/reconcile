import type { Database } from "src/database/database";
import type { RelativePath } from "src/database/document-metadata";
import type { SyncService } from "src/services/sync-service";
import { unlockDocument, waitForDocumentLock } from "./locks";
import { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history";

export async function syncLocallyDeletedFile({
	database,
	syncServer,
	history,
	relativePath,
}: {
	database: Database;
	syncServer: SyncService;
	history: SyncHistory;
	relativePath: RelativePath;
}): Promise<void> {
	if (!database.getSettings().isSyncEnabled) {
		Logger.getInstance().info(
			`Syncing is disabled, not syncing ${relativePath}`
		);
		return;
	}
	Logger.getInstance().debug(`Syncing ${relativePath}`);

	await waitForDocumentLock(relativePath);

	try {
		const metadata = database.getDocument(relativePath);
		if (!metadata) {
			history.addHistoryEntry({
				status: SyncStatus.NO_OP,
				relativePath,
				message: `Locally deleted file hasn't been uploaded yet, so there's no need to delete it on the remote server`,
				type: SyncType.DELETE,
			});
			return;
		}

		await syncServer.delete({
			documentId: metadata.documentId,
			relativePath,
			// We got the event now, so it must have been deleted just now
			createdDate: new Date(),
		});

		history.addHistoryEntry({
			status: SyncStatus.SUCCESS,
			source: SyncSource.PUSH,
			relativePath,
			message: `Successfully deleted locally deleted file on the remote server`,
			type: SyncType.DELETE,
		});

		await database.removeDocument(relativePath);
	} catch (e) {
		history.addHistoryEntry({
			status: SyncStatus.ERROR,
			relativePath,
			message: `Failed to remotely delete locally deleted file: ${e}`,
			type: SyncType.DELETE,
		});
		throw e;
	} finally {
		unlockDocument(relativePath);
	}
}
