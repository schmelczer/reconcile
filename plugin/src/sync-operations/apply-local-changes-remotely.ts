import { Database } from "../database/database";
import { SyncService } from "../services/sync_service";
import { Logger } from "../logger";
import { FileOperations } from "../file-operations/file-operations";
import { syncLocallyCreatedFile } from "./sync-locally-created-file";
import { EMPTY_HASH, hash } from "src/utils/hash";
import { syncLocallyUpdatedFile } from "./sync-locally-updated-file";
import { syncLocallyDeletedFile } from "./sync-locally-deleted-file";
import { Notice } from "obsidian";
import PQueue from "p-queue";

let isRunning = false;

export interface Progress {
	processedFiles: number;
	totalFiles: number;
}

export async function applyLocalChangesRemotely(
	database: Database,
	syncServer: SyncService,
	operations: FileOperations
) {
	console.log("applyLocalChangesRemotely");
	if (isRunning) {
		Logger.getInstance().info("Push sync already in progress, skipping");
		return;
	}

	let tasks: Promise<void>[] = [];

	const allLocalFiles = await operations.listAllFiles();
	console.log(allLocalFiles);
	const deletedFiles = [...database.getDocuments().entries()].filter(
		([path, _]) => !allLocalFiles.includes(path)
	);

	console.log(deletedFiles);

	const promiseQueue = new PQueue({
		concurrency: 1,
	});

	await Promise.all(
		allLocalFiles.map((path) =>
			promiseQueue.add(async () => {
				const syncedState = database.getDocument(path);
				if (!syncedState) {
					Logger.getInstance().info(
						`Document ${path} not found in database`
					);
					const contentHash = hash(await operations.read(path));
					if (contentHash != EMPTY_HASH) {
						const match = deletedFiles.find(
							([path, doc]) => doc.hash === contentHash
						);
						if (match) {
							const oldPath = match[0];
							Logger.getInstance().info(
								`Document ${path} found remotely under a different path (${oldPath}), moving`
							);
							tasks.push(
								syncLocallyUpdatedFile({
									database,
									syncServer,
									operations,
									oldPath,
									filePath: path,
									updateTime:
										await operations.getModificationTime(
											path
										),
								})
							);
							deletedFiles.remove(match);
							return;
						}
					}
					tasks.push(
						syncLocallyCreatedFile({
							database,
							syncServer,
							operations,
							updateTime: await operations.getModificationTime(
								path
							),
							filePath: path,
						})
					);
					return;
				}

				const content = await operations.read(path);
				if (syncedState.hash !== hash(content)) {
					Logger.getInstance().info(
						`Document ${path} has local changes, updating`
					);
					tasks.push(
						syncLocallyUpdatedFile({
							database,
							syncServer,
							operations,
							filePath: path,
							updateTime: await operations.getModificationTime(
								path
							),
						})
					);
					return;
				}
			})
		)
	);

	deletedFiles.forEach(([relativePath, _]) => {
		Logger.getInstance().info(
			`Document ${relativePath} deleted locally, deleting`
		);
		tasks.push(
			syncLocallyDeletedFile({
				database,
				syncServer,
				relativePath,
			})
		);
	});

	await Promise.all(tasks);

	new Notice("Local changes synced remotely");
}
