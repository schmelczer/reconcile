import type {
	Database,
	DocumentMetadata,
	RelativePath
} from "../persistence/database";

import type { SyncService } from "src/services/sync-service";
import { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history";
import { unlockDocument, waitForDocumentLock } from "./document-lock";
import PQueue from "p-queue";
import { EMPTY_HASH, hash } from "src/utils/hash";
import type { components } from "src/services/types";
import { deserialize } from "src/utils/deserialize";
import type { Settings } from "src/persistence/settings";
import { FileOperations } from "src/file-operations/file-operations";
import { findMatchingFileBasedOnHash } from "src/utils/find-matching-file-based-on-hash";

export class Syncer {
	private readonly remainingOperationsListeners: ((
		remainingOperations: number
	) => void)[] = [];

	private readonly syncQueue: PQueue;

	private runningScheduleSyncForOfflineChanges: Promise<void> | undefined =
		undefined;
	private runningApplyRemoteChangesLocally: Promise<void> | undefined =
		undefined;

	public constructor(
		private readonly logger: Logger,
		private readonly database: Database,
		private readonly settings: Settings,
		private readonly syncService: SyncService,
		private readonly operations: FileOperations,
		private readonly history: SyncHistory
	) {
		this.syncQueue = new PQueue({
			concurrency: settings.getSettings().syncConcurrency
		});

		settings.addOnSettingsChangeHandlers((newSettings) => {
			this.syncQueue.concurrency = newSettings.syncConcurrency;
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

	public waitForSyncQueue(): Promise<void> {
		return this.syncQueue.onEmpty();
	}

	public async syncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncLocallyDeletedFile(relativePath)
		);
	}

	private async syncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"]
	): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncRemotelyUpdatedFile(remoteVersion)
		);
	}

	public async scheduleSyncForOfflineChanges(): Promise<void> {
		if (!this.settings.getSettings().isSyncEnabled) {
			this.logger.debug(
				`Syncing is disabled, not uploading local changes`
			);
			return;
		}

		if (this.runningScheduleSyncForOfflineChanges != null) {
			this.logger.debug("Uploading local changes is already in progress");
			return this.runningScheduleSyncForOfflineChanges;
		}

		try {
			this.runningScheduleSyncForOfflineChanges =
				this.internalScheduleSyncForOfflineChanges();
			await this.runningScheduleSyncForOfflineChanges;
			this.logger.info(`All local changes have been applied remotely`);
		} catch (e) {
			this.logger.error(
				`Not all local changes have been applied remotely: ${e}`
			);
			throw e;
		} finally {
			this.runningScheduleSyncForOfflineChanges = undefined;
		}
	}

	private async internalScheduleSyncForOfflineChanges(): Promise<void> {
		const allLocalFiles = await this.operations.listAllFiles();
		let locallyDeletedFiles = [
			...this.database.getDocuments().entries()
		].filter(([path, _]) => !allLocalFiles.includes(path));

		await Promise.all(
			allLocalFiles.map(async (relativePath) =>
				this.syncQueue.add(async () => {
					const metadata = this.database.getDocument(relativePath);

					// If there's no metadata, it must be a new file
					if (!metadata) {
						// Perhaps the file has been moved. Let's check by looking at the deleted files
						const contentBytes =
							await this.operations.read(relativePath);
						const contentHash = hash(contentBytes);

						const originalFile = findMatchingFileBasedOnHash(
							contentHash,
							locallyDeletedFiles
						);
						if (originalFile !== undefined) {
							// `originalFile` hasn't been deleted but it got moved instead
							locallyDeletedFiles = locallyDeletedFiles.filter(
								(item) => item != originalFile
							);

							this.logger.debug(
								`Document ${relativePath} was not found under its current path in the database but was found under a different path ${originalFile[0]}, scheduling sync to move it`
							);
							return this.internalSyncLocallyUpdatedFile({
								oldPath: originalFile[0],
								relativePath: relativePath,
								updateTime:
									await this.operations.getModificationTime(
										relativePath
									),
								optimisations: {
									contentBytes,
									contentHash
								}
							});
						}

						this.logger.debug(
							`Document ${relativePath} not found in database, scheduling sync to create it`
						);
						return this.internalSyncLocallyCreatedFile(
							relativePath,
							await this.operations.getModificationTime(
								relativePath
							)
						);
					}

					this.logger.debug(
						`Document ${relativePath} has been updated locally, scheduling sync to update it`
					);
					return this.internalSyncLocallyUpdatedFile({
						relativePath,
						updateTime:
							await this.operations.getModificationTime(
								relativePath
							)
					});
				})
			)
		);

		await Promise.all(
			locallyDeletedFiles.map(async ([relativePath, _]) => {
				this.logger.debug(
					`Document ${relativePath} has been deleted locally, scheduling sync to delete it`
				);

				if (await this.operations.exists(relativePath)) {
					this.logger.debug(
						`Document ${relativePath} actually exists locally, skipping`
					);
					return Promise.resolve();
				}

				// We're outside of the pqueue, so we need to call the public wrapper
				return this.syncLocallyDeletedFile(relativePath);
			})
		);
	}

	public async applyRemoteChangesLocally(): Promise<void> {
		if (!this.settings.getSettings().isSyncEnabled) {
			this.logger.debug(
				`Syncing is disabled, not fetching remote changes`
			);
			return;
		}

		if (this.runningApplyRemoteChangesLocally != null) {
			this.logger.debug(
				"Applying remote changes locally is already in progress"
			);
			return this.runningApplyRemoteChangesLocally;
		}

		try {
			this.runningApplyRemoteChangesLocally =
				this.internalApplyRemoteChangesLocally();
			await this.runningApplyRemoteChangesLocally;
			this.logger.info("All remote changes have been applied locally");
		} catch (e) {
			this.logger.error(`Failed to apply remote changes locally: ${e}`);
			throw e;
		} finally {
			this.runningApplyRemoteChangesLocally = undefined;
		}
	}

	private async internalApplyRemoteChangesLocally(): Promise<void> {
		const remote = await this.syncService.getAll(
			this.database.getLastSeenUpdateId()
		);

		if (remote.latestDocuments.length === 0) {
			this.logger.debug("No remote changes to apply");
			return;
		}

		this.logger.info("Applying remote changes locally");

		await Promise.all(
			remote.latestDocuments.map(async (remoteDocument) =>
				this.syncRemotelyUpdatedFile(remoteDocument)
			)
		);

		const lastSeenUpdateId = this.database.getLastSeenUpdateId();
		if (
			lastSeenUpdateId === undefined ||
			remote.lastUpdateId > lastSeenUpdateId
		) {
			await this.database.setLastSeenUpdateId(remote.lastUpdateId);
		}
	}

	public async reset(): Promise<void> {
		this.syncQueue.clear();
		await this.syncQueue.onEmpty();
		this.remainingOperationsListeners.forEach((listener) => {
			listener(0);
		});
	}

	private async internalSyncLocallyCreatedFile(
		relativePath: RelativePath,
		updateTime: Date,
		optimisations?: {
			contentBytes?: Uint8Array;
			contentHash?: string;
		}
	): Promise<void> {
		await this.executeWhileHoldingFileLock(
			relativePath,
			SyncType.CREATE,
			SyncSource.PUSH,
			async () => {
				if (
					(await this.operations.getFileSize(relativePath)) /
						1024 /
						1024 >
					this.settings.getSettings().maxFileSizeMB
				) {
					this.history.addHistoryEntry({
						status: SyncStatus.ERROR,
						relativePath,
						message: `File size exceeds the maximum file size limit of ${
							this.settings.getSettings().maxFileSizeMB
						}MB`,
						type: SyncType.CREATE
					});
					return;
				}

				const contentBytes =
					optimisations?.contentBytes ??
					(await this.operations.read(relativePath));
				let contentHash =
					optimisations?.contentHash ?? hash(contentBytes);

				const localMetadata = this.database.getDocument(relativePath);
				if (localMetadata) {
					this.logger.debug(
						`Document metadata already exists for ${relativePath}, it must have been downloaded from the server`
					);

					if (localMetadata.hash === contentHash) {
						this.history.addHistoryEntry({
							status: SyncStatus.NO_OP,
							relativePath,
							message: `File hash matches with last synced version, no need to sync`,
							type: SyncType.UPDATE
						});
						return;
					}
				}

				const response = await this.syncService.create({
					relativePath,
					contentBytes,
					createdDate: updateTime
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PUSH,
					relativePath,
					message: `Successfully uploaded locally created file`,
					type: SyncType.CREATE
				});

				if (response.type === "MergingUpdate") {
					const responseBytes = deserialize(response.contentBase64);
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
						type: SyncType.UPDATE
					});
				}

				await this.database.setDocument({
					documentId: response.documentId,
					relativePath: response.relativePath,
					parentVersionId: response.vaultUpdateId,
					hash: contentHash
				});

				await this.tryIncrementVaultUpdateId(response.vaultUpdateId);
			}
		);
	}

	private async internalSyncLocallyUpdatedFile({
		oldPath,
		relativePath,
		updateTime,
		optimisations
	}: {
		oldPath?: RelativePath;
		relativePath: RelativePath;
		updateTime: Date;
		optimisations?: {
			contentBytes?: Uint8Array;
			contentHash?: string;
		};
	}): Promise<void> {
		await this.executeWhileHoldingFileLock(
			relativePath,
			SyncType.UPDATE,
			SyncSource.PUSH,
			async () => {
				this.logger.debug(
					`Renaming? oldPath ${oldPath} relativePath ${relativePath}`
				);

				const localMetadata = this.database.getDocument(
					oldPath ?? relativePath
				);

				if (!localMetadata) {
					if (this.database.getDocument(relativePath)) {
						this.history.addHistoryEntry({
							status: SyncStatus.NO_OP,
							relativePath,
							message: `The renaming doesn't require a sync because it must have been pulled from remote`,
							type: SyncType.UPDATE
						});
						return;
					}

					this.logger.debug(
						`Document metadata doesn't exist for ${relativePath}, it must have been already deleted`
					);
					return;
				}

				if (
					(await this.operations.getFileSize(relativePath)) /
						1024 /
						1024 >
					this.settings.getSettings().maxFileSizeMB
				) {
					this.history.addHistoryEntry({
						status: SyncStatus.ERROR,
						relativePath,
						message: `File size exceeds the maximum file size limit of ${
							this.settings.getSettings().maxFileSizeMB
						}MB`,
						type: SyncType.CREATE
					});
					return;
				}

				const contentBytes =
					optimisations?.contentBytes ??
					(await this.operations.read(relativePath));

				let contentHash =
					optimisations?.contentHash ?? hash(contentBytes);

				if (
					localMetadata.hash === contentHash &&
					oldPath === undefined
				) {
					this.history.addHistoryEntry({
						status: SyncStatus.NO_OP,
						relativePath,
						message: `File hash matches with last synced version, no need to sync`,
						type: SyncType.UPDATE
					});
					return;
				}

				const response = await this.syncService.put({
					documentId: localMetadata.documentId,
					parentVersionId: localMetadata.parentVersionId,
					relativePath,
					contentBytes,
					createdDate: updateTime
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PUSH,
					relativePath,
					message: `Successfully uploaded locally updated file to the remote server`,
					type: SyncType.UPDATE
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
						type: SyncType.DELETE
					});

					return;
				}

				if (response.relativePath != relativePath) {
					await waitForDocumentLock(response.relativePath);
				}

				try {
					if (response.relativePath != relativePath) {
						await this.operations.move(
							relativePath,
							response.relativePath
						);
					}

					if (response.type === "MergingUpdate") {
						const responseBytes = deserialize(
							response.contentBase64
						);
						contentHash = hash(responseBytes);

						await this.operations.write(
							response.relativePath,
							contentBytes,
							responseBytes
						);

						this.history.addHistoryEntry({
							status: SyncStatus.SUCCESS,
							source: SyncSource.PULL,
							relativePath,
							message: `The file we updated had been updated remotely, so we downloaded the merged version`,
							type: SyncType.UPDATE
						});
					}

					await this.database.moveDocument({
						documentId: localMetadata.documentId,
						oldRelativePath: oldPath ?? relativePath,
						relativePath: response.relativePath,
						parentVersionId: response.vaultUpdateId,
						hash: contentHash
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
						type: SyncType.DELETE
					});
					return;
				}

				await this.syncService.delete({
					documentId: localMetadata.documentId,
					relativePath,
					createdDate: new Date() // We got the event now, so it must have been deleted just now
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PUSH,
					relativePath,
					message: `Successfully deleted locally deleted file on the remote server`,
					type: SyncType.DELETE
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
							type: SyncType.DELETE
						});
						return;
					}

					const content = (
						await this.syncService.get({
							documentId: remoteVersion.documentId
						})
					).contentBase64;
					const contentBytes = deserialize(content);

					await this.operations.create(
						remoteVersion.relativePath,
						contentBytes
					);
					await this.database.setDocument({
						documentId: remoteVersion.documentId,
						relativePath: remoteVersion.relativePath,
						parentVersionId: remoteVersion.vaultUpdateId,
						hash: hash(contentBytes)
					});
					this.history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						source: SyncSource.PULL,
						relativePath: remoteVersion.relativePath,
						message: `Successfully downloaded remote file which hadn't existed locally`,
						type: SyncType.CREATE
					});
					return;
				}

				const [relativePath, metadata] = localMetadata;
				if (metadata.parentVersionId === remoteVersion.vaultUpdateId) {
					this.logger.debug(
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
							type: SyncType.DELETE
						});
					} else {
						const currentContent =
							await this.operations.read(relativePath);
						const currentHash = hash(currentContent);

						if (currentHash !== metadata.hash) {
							this.logger.info(
								`Document ${relativePath} has been updated both remotely and locally, letting the local file update event handle it`
							);
							return;
						}

						const content = (
							await this.syncService.get({
								documentId: remoteVersion.documentId
							})
						).contentBase64;
						const contentBytes = deserialize(content);
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
							hash: contentHash
						});

						this.history.addHistoryEntry({
							status: SyncStatus.SUCCESS,
							source: SyncSource.PULL,
							relativePath: remoteVersion.relativePath,
							message: `Successfully updated remotely updated file locally`,
							type: SyncType.UPDATE
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
		if (!this.settings.getSettings().isSyncEnabled) {
			this.logger.info(
				`Syncing is disabled, not syncing ${relativePath}`
			);
			return;
		}
		if (!this.operations.isFileEligibleForSync(relativePath)) {
			this.logger.info(
				`File ${relativePath} is not eligible for syncing`
			);
			return;
		}
		this.logger.debug(
			`Syncing ${relativePath} (${syncSource} - ${syncType})`
		);

		await waitForDocumentLock(relativePath);
		try {
			await fn();
		} catch (e) {
			this.history.addHistoryEntry({
				status: SyncStatus.ERROR,
				relativePath,
				message: `Failed to ${syncSource.toLocaleLowerCase()} file ${e} when trying to ${syncType.toLocaleLowerCase()} it`,
				type: syncType,
				source: syncSource
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
}
