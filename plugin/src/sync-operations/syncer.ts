import type { Database } from "src/database/database";
import type {
	DocumentMetadata,
	RelativePath,
} from "src/database/document-metadata";
import type { FileOperations } from "src/file-operations/file-operations";
import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import type { SyncService } from "src/services/sync-service";
import { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history";
import { unlockDocument, waitForDocumentLock } from "./document-lock";
import PQueue from "p-queue";
import { EMPTY_HASH, hash } from "src/utils/hash";
import type { components } from "src/services/types.js";

export class Syncer {
	private readonly remainingOperationsListeners: ((
		remainingOperations: number
	) => void)[] = [];

	private readonly syncQueue: PQueue;

	private isRunningOfflineSync = false;

	public constructor(
		private readonly database: Database,
		private readonly syncService: SyncService,
		private readonly operations: FileOperations,
		private readonly history: SyncHistory
	) {
		this.syncQueue = new PQueue({
			concurrency: database.getSettings().syncConcurrency,
		});

		database.addOnSettingsChangeHandlers((settings) => {
			this.syncQueue.concurrency = settings.syncConcurrency;
		});

		this.syncQueue.on("active", () => {
			this.emitRemainingOperationsChange(this.syncQueue.size);
		});
	}

	public addRemainingOperationsListener(
		listener: (remainingOperations: number) => void
	): void {
		this.remainingOperationsListeners.push(listener);
	}

	public async syncLocallyCreatedFile(
		relativePath: RelativePath,
		updateTime: Date
	): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncLocallyCreatedFile(relativePath, updateTime)
		);
	}

	public async syncLocallyUpdatedFile(args: {
		oldPath?: RelativePath;
		relativePath: RelativePath;
		updateTime: Date;
	}): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncLocallyUpdatedFile(args)
		);
	}

	public async syncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncLocallyDeletedFile(relativePath)
		);
	}

	public async syncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"]
	): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncRemotelyUpdatedFile(remoteVersion)
		);
	}

	public async scheduleSyncForOfflineChanges(): Promise<void> {
		if (this.isRunningOfflineSync) {
			Logger.getInstance().warn(
				"Uploading local changes is already in progress, skipping"
			);
			return;
		}

		if (!this.database.getSettings().isSyncEnabled) {
			Logger.getInstance().debug(
				`Syncing is disabled, not uploading local changes`
			);
			return;
		}

		this.isRunningOfflineSync = true;

		try {
			const allLocalFiles = await this.operations.listAllFiles();
			const locallyDeletedFiles = [
				...this.database.getDocuments().entries(),
			].filter(([path, _]) => !allLocalFiles.includes(path));

			await Promise.all(
				allLocalFiles.map(async (relativePath) =>
					this.syncQueue.add(async () => {
						const metadata =
							this.database.getDocument(relativePath);

						// If there's no metadata, it must be a new file
						if (!metadata) {
							// Perhaps the file has been moved. Let's check by looking at the deleted files
							const originalFile =
								await this.findMatchingFileBasedOnHash(
									relativePath,
									locallyDeletedFiles
								);
							if (originalFile !== undefined) {
								// `originalFile` hasn't been deleted but it got moved instead
								locallyDeletedFiles.remove(originalFile);

								Logger.getInstance().debug(
									`Document ${relativePath} was not found under its current path in the database but was found under a different path ${originalFile[0]}, scheduling sync to move it`
								);
								return this.internalSyncLocallyUpdatedFile({
									oldPath: originalFile[0],
									relativePath: relativePath,
									updateTime:
										await this.operations.getModificationTime(
											relativePath
										),
								});
							}

							Logger.getInstance().debug(
								`Document ${relativePath} not found in database, scheduling sync to create it`
							);
							return this.internalSyncLocallyCreatedFile(
								relativePath,
								await this.operations.getModificationTime(
									relativePath
								)
							);
						}

						Logger.getInstance().debug(
							`Document ${relativePath} has been updated locally, scheduling sync to update it`
						);
						return this.internalSyncLocallyUpdatedFile({
							relativePath,
							updateTime:
								await this.operations.getModificationTime(
									relativePath
								),
						});
					})
				)
			);

			await Promise.all(
				locallyDeletedFiles.map(async ([relativePath, _]) => {
					Logger.getInstance().debug(
						`Document ${relativePath} has been deleted locally, scheduling sync to delete it`
					);

					return this.internalSyncLocallyDeletedFile(relativePath);
				})
			);

			Logger.getInstance().info(
				`All local changes have been applied remotely`
			);
		} catch (e) {
			Logger.getInstance().error(
				`Not all local changes have been applied remotely: ${e}`
			);
		} finally {
			this.isRunningOfflineSync = false;
		}
	}

	public async reset(): Promise<void> {
		this.syncQueue.clear();
		await this.syncQueue.onEmpty();
		await this.database.resetSyncState();
		this.history.reset();
		this.remainingOperationsListeners.forEach((listener) => {
			listener(0);
		});
	}

	private async internalSyncLocallyCreatedFile(
		relativePath: RelativePath,
		updateTime: Date
	): Promise<void> {
		await this.executeWhileHoldingFileLock(
			relativePath,
			SyncType.CREATE,
			SyncSource.PUSH,
			async () => {
				const contentBytes = await this.operations.read(relativePath);
				let contentHash = hash(contentBytes);

				const localMetadata = this.database.getDocument(relativePath);
				if (localMetadata) {
					Logger.getInstance().debug(
						`Document metadata already exists for ${relativePath}, it must have been downloaded from the server`
					);

					if (localMetadata.hash === contentHash) {
						this.history.addHistoryEntry({
							status: SyncStatus.NO_OP,
							relativePath,
							message: `File hash matches with last synced version, no need to sync`,
							type: SyncType.UPDATE,
						});
						return;
					}
				}

				const response = await this.syncService.create({
					relativePath,
					contentBytes,
					createdDate: updateTime,
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PUSH,
					relativePath,
					message: `Successfully uploaded locally created file`,
					type: SyncType.CREATE,
				});

				if (response.type === "MergingUpdate") {
					const responseBytes = lib.base64ToBytes(
						response.contentBase64
					);
					contentHash = hash(responseBytes);

					await this.operations.write(
						relativePath,
						contentBytes,
						responseBytes
					);
					this.history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						source: SyncSource.PULL,
						relativePath,
						message: `The file we created locally has already existed remotely, so we have merged them`,
						type: SyncType.UPDATE,
					});
				}

				await this.database.setDocument({
					documentId: response.documentId,
					relativePath: response.relativePath,
					parentVersionId: response.vaultUpdateId,
					hash: contentHash,
				});

				await this.tryIncrementVaultUpdateId(response.vaultUpdateId);
			}
		);
	}

	private async internalSyncLocallyUpdatedFile({
		oldPath,
		relativePath,
		updateTime,
	}: {
		oldPath?: RelativePath;
		relativePath: RelativePath;
		updateTime: Date;
	}): Promise<void> {
		await this.executeWhileHoldingFileLock(
			relativePath,
			SyncType.UPDATE,
			SyncSource.PUSH,
			async () => {
				const localMetadata = this.database.getDocument(
					oldPath ?? relativePath
				);
				if (!localMetadata) {
					if (this.database.getDocument(relativePath)) {
						this.history.addHistoryEntry({
							status: SyncStatus.NO_OP,
							relativePath,
							message: `The renaming doesn't require a sync because it must have been pulled from remote`,
							type: SyncType.UPDATE,
						});
						return;
					}

					throw new Error(
						`Document metadata not found for ${relativePath}. This implies a corrupt local database. Consider resetting the plugin's sync history.`
					);
				}

				const contentBytes = await this.operations.read(relativePath);
				let contentHash = hash(contentBytes);

				if (
					localMetadata.hash === contentHash &&
					oldPath === undefined
				) {
					this.history.addHistoryEntry({
						status: SyncStatus.NO_OP,
						relativePath,
						message: `File hash matches with last synced version, no need to sync`,
						type: SyncType.UPDATE,
					});
					return;
				}

				const response = await this.syncService.put({
					documentId: localMetadata.documentId,
					parentVersionId: localMetadata.parentVersionId,
					relativePath,
					contentBytes,
					createdDate: updateTime,
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PUSH,
					relativePath,
					message: `Successfully uploaded locally updated file to the remote server`,
					type: SyncType.UPDATE,
				});

				if (response.isDeleted) {
					await this.operations.remove(oldPath ?? relativePath);
					await this.database.removeDocument(oldPath ?? relativePath);
					await this.tryIncrementVaultUpdateId(
						response.vaultUpdateId
					);

					this.history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						source: SyncSource.PULL,
						relativePath,
						message:
							"The file we tried to update had been deleted remotely, therefore, we have deleted it locally",
						type: SyncType.DELETE,
					});

					return;
				}

				if (response.relativePath != relativePath) {
					await waitForDocumentLock(response.relativePath);
				}

				try {
					if (response.relativePath != relativePath) {
						await this.operations.move(
							oldPath ?? relativePath,
							response.relativePath
						);
					}

					if (response.type === "MergingUpdate") {
						const responseBytes = lib.base64ToBytes(
							response.contentBase64
						);
						contentHash = hash(responseBytes);
						await this.operations.write(
							response.relativePath,
							contentBytes,
							responseBytes
						);
					}

					this.history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						source: SyncSource.PULL,
						relativePath,
						message: `The file we updated had been updated remotely, so we downloaded the merged version`,
						type: SyncType.UPDATE,
					});

					await this.database.moveDocument({
						documentId: localMetadata.documentId,
						oldRelativePath: oldPath ?? relativePath,
						relativePath: response.relativePath,
						parentVersionId: response.vaultUpdateId,
						hash: contentHash,
					});

					await this.tryIncrementVaultUpdateId(
						response.vaultUpdateId
					);
				} finally {
					if (response.relativePath != relativePath) {
						unlockDocument(response.relativePath);
					}
				}
			}
		);
	}

	private async internalSyncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		await this.executeWhileHoldingFileLock(
			relativePath,
			SyncType.DELETE,
			SyncSource.PUSH,
			async () => {
				const localMetadata = this.database.getDocument(relativePath);
				if (!localMetadata) {
					this.history.addHistoryEntry({
						status: SyncStatus.NO_OP,
						relativePath,
						message: `Locally deleted file hasn't been uploaded yet, so there's no need to delete it on the remote server`,
						type: SyncType.DELETE,
					});
					return;
				}

				await this.syncService.delete({
					documentId: localMetadata.documentId,
					relativePath,
					createdDate: new Date(), // We got the event now, so it must have been deleted just now
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PUSH,
					relativePath,
					message: `Successfully deleted locally deleted file on the remote server`,
					type: SyncType.DELETE,
				});

				await this.database.removeDocument(relativePath);
			}
		);
	}

	private async internalSyncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"]
	): Promise<void> {
		await this.executeWhileHoldingFileLock(
			remoteVersion.relativePath,
			SyncType.UPDATE,
			SyncSource.PULL,
			async () => {
				const localMetadata = this.database.getDocumentByDocumentId(
					remoteVersion.documentId
				);

				if (!localMetadata) {
					if (remoteVersion.isDeleted) {
						this.history.addHistoryEntry({
							status: SyncStatus.NO_OP,
							source: SyncSource.PULL,
							relativePath: remoteVersion.relativePath,
							message: `Remotely deleted file hasn't been synced yet, so there's no need to delete it locally`,
							type: SyncType.DELETE,
						});
						return;
					}

					const content = (
						await this.syncService.get({
							documentId: remoteVersion.documentId,
						})
					).contentBase64;
					const contentBytes = lib.base64ToBytes(content);

					await this.operations.create(
						remoteVersion.relativePath,
						contentBytes
					);
					await this.database.setDocument({
						documentId: remoteVersion.documentId,
						relativePath: remoteVersion.relativePath,
						parentVersionId: remoteVersion.vaultUpdateId,
						hash: hash(contentBytes),
					});
					this.history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						source: SyncSource.PULL,
						relativePath: remoteVersion.relativePath,
						message: `Successfully downloaded remote file which hasn't existed locally`,
						type: SyncType.CREATE,
					});
					return;
				}

				const [relativePath, metadata] = localMetadata;
				if (metadata.parentVersionId === remoteVersion.vaultUpdateId) {
					Logger.getInstance().debug(
						`Document ${relativePath} is already up to date`
					);
					return;
				}

				if (relativePath !== remoteVersion.relativePath) {
					await waitForDocumentLock(relativePath);
				}
				try {
					if (remoteVersion.isDeleted) {
						await this.operations.remove(relativePath);
						await this.database.removeDocument(relativePath);

						this.history.addHistoryEntry({
							status: SyncStatus.SUCCESS,
							source: SyncSource.PULL,
							relativePath: remoteVersion.relativePath,
							message: `Successfully deleted remotely deleted file locally`,
							type: SyncType.DELETE,
						});
					} else {
						const currentContent = await this.operations.read(
							relativePath
						);
						const currentHash = hash(currentContent);

						if (currentHash !== metadata.hash) {
							Logger.getInstance().info(
								`Document ${relativePath} has been updated both remotely and locally, letting the local file update event handle it`
							);
							return;
						}

						const content = (
							await this.syncService.get({
								documentId: remoteVersion.documentId,
							})
						).contentBase64;
						const contentBytes = lib.base64ToBytes(content);
						const contentHash = hash(contentBytes);

						if (relativePath !== remoteVersion.relativePath) {
							await this.operations.move(
								relativePath,
								remoteVersion.relativePath
							);
						}

						await this.operations.write(
							remoteVersion.relativePath,
							currentContent,
							contentBytes
						);
						await this.database.moveDocument({
							documentId: remoteVersion.documentId,
							oldRelativePath: relativePath,
							relativePath: remoteVersion.relativePath,
							parentVersionId: remoteVersion.vaultUpdateId,
							hash: contentHash,
						});

						this.history.addHistoryEntry({
							status: SyncStatus.SUCCESS,
							source: SyncSource.PULL,
							relativePath: remoteVersion.relativePath,
							message: `Successfully updated remotely updated file locally`,
							type: SyncType.UPDATE,
						});
					}
				} finally {
					if (relativePath !== remoteVersion.relativePath) {
						unlockDocument(relativePath);
					}
				}
			}
		);
	}

	private async executeWhileHoldingFileLock(
		relativePath: RelativePath,
		syncType: SyncType,
		syncSource: SyncSource,
		fn: () => Promise<void>
	): Promise<void> {
		if (!this.database.getSettings().isSyncEnabled) {
			Logger.getInstance().info(
				`Syncing is disabled, not syncing ${relativePath}`
			);
			return;
		}
		Logger.getInstance().debug(`Syncing ${relativePath}`);

		await waitForDocumentLock(relativePath);
		try {
			await fn();
		} catch (e) {
			this.history.addHistoryEntry({
				status: SyncStatus.ERROR,
				relativePath,
				message: `Failed to ${syncSource.toLocaleLowerCase()} file ${e} when trying to ${syncType.toLocaleLowerCase()} it`,
				type: syncType,
				source: syncSource,
			});
			throw e;
		} finally {
			unlockDocument(relativePath);
		}
	}

	private emitRemainingOperationsChange(remainingOperations: number): void {
		this.remainingOperationsListeners.forEach((listener) => {
			listener(remainingOperations);
		});
	}

	private async tryIncrementVaultUpdateId(
		responseVaultUpdateId: number
	): Promise<void> {
		if (this.database.getLastSeenUpdateId() === responseVaultUpdateId - 1) {
			await this.database.setLastSeenUpdateId(responseVaultUpdateId);
		}
	}

	private async findMatchingFileBasedOnHash(
		filePath: RelativePath,
		candidates: [RelativePath, DocumentMetadata][]
	): Promise<[RelativePath, DocumentMetadata] | undefined> {
		const contentHash = hash(await this.operations.read(filePath));

		if (contentHash != EMPTY_HASH) {
			return undefined;
		}

		return candidates.find(
			([_, document]) => document.hash === contentHash
		);
	}
}
