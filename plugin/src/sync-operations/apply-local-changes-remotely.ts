import type { Database } from "../database/database";
import type { SyncService } from "../services/sync-service";
import type { FileOperations } from "../file-operations/file-operations";
import { syncLocallyCreatedFile } from "./sync-locally-created-file";
import { EMPTY_HASH, hash } from "src/utils/hash";
import { syncLocallyUpdatedFile } from "./sync-locally-updated-file";
import { syncLocallyDeletedFile } from "./sync-locally-deleted-file";
import { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";

let isRunning = false;

export async function applyLocalChangesRemotely({
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
	if (isRunning) {
		Logger.getInstance().debug(
			"Uploading local changes is already in progress, skipping"
		);
		return;
	}

	isRunning = true;
	try {
		const tasks: Promise<void>[] = [];

		const allLocalFiles = await operations.listAllFiles();
		const locallyDeletedFiles = [
			...database.getDocuments().entries(),
		].filter(([path, _]) => !allLocalFiles.includes(path));

		await Promise.all(
			allLocalFiles.map(async (path) => {
				const metadata = database.getDocument(path);
				if (!metadata) {
					const contentHash = hash(await operations.read(path));
					const match = locallyDeletedFiles.find(
						([_, document]) => document.hash === contentHash
					);

					if (contentHash != EMPTY_HASH && match) {
						locallyDeletedFiles.remove(match);

						Logger.getInstance().debug(
							`Document ${path} not found in database but found under a different path ${match[0]}, scheduling sync to update it`
						);
						return syncLocallyUpdatedFile({
							database,
							syncServer,
							operations,
							history,
							oldPath: match[0],
							relativePath: path,
							updateTime: await operations.getModificationTime(
								path
							),
						});
					}

					Logger.getInstance().debug(
						`Document ${path} not found in database, scheduling sync to create it`
					);
					return syncLocallyCreatedFile({
						database,
						syncServer,
						operations,
						history,
						updateTime: await operations.getModificationTime(path),
						relativePath: path,
					});
				}

				const content = await operations.read(path);
				if (metadata.hash !== hash(content)) {
					Logger.getInstance().debug(
						`Document ${path} has been updated locally, scheduling sync to update it`
					);
					return syncLocallyUpdatedFile({
						database,
						syncServer,
						operations,
						history,
						relativePath: path,
						updateTime: await operations.getModificationTime(path),
					});
				}

				return Promise.resolve();
			})
		);

		tasks.push(
			...locallyDeletedFiles.map(async ([relativePath, _]) => {
				Logger.getInstance().debug(
					`Document ${relativePath} has been deleted locally, scheduling sync to delete it`
				);

				return syncLocallyDeletedFile({
					database,
					syncServer,
					history,
					relativePath,
				});
			})
		);

		try {
			await Promise.all(tasks);
			Logger.getInstance().info(
				`All local changes have been applied remotely`
			);
			return;
		} catch {
			await Promise.allSettled(tasks);
			Logger.getInstance().error(
				`Not all local changes have been applied remotely`
			);
		}
	} finally {
		isRunning = false;
	}
}
