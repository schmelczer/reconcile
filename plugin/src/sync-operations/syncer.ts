import type { Database } from "src/database/database";
import type { RelativePath } from "src/database/document-metadata";
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
	private readonly database: Database;
	private readonly syncServer: SyncService;
	private readonly operations: FileOperations;
	private readonly history: SyncHistory;

	private isRunningOfflineSync = false;

	private readonly offlineSyncQueue: PQueue;
	private readonly fileSyncQueue: PQueue;
	private readonly remainingOperationsListeners: ((
		remainingOperations: number
	) => void)[] = [];

	public constructor({
		database,
		syncServer,
		operations,
		history,
	}: {
		database: Database;
		syncServer: SyncService;
		operations: FileOperations;
		history: SyncHistory;
	}) {
		this.database = database;
		this.syncServer = syncServer;
		this.operations = operations;
		this.history = history;

		this.fileSyncQueue = new PQueue({
			concurrency: database.getSettings().syncConcurrency,
		});
		this.offlineSyncQueue = new PQueue({
			concurrency: database.getSettings().syncConcurrency,
		});

		database.addOnSettingsChangeHandlers((settings) => {
			this.fileSyncQueue.concurrency = settings.syncConcurrency;
			this.offlineSyncQueue.concurrency = settings.syncConcurrency;
		});

		this.fileSyncQueue.on("active", () => {
			this.emitRemainingOperationsChange(
				this.fileSyncQueue.size + this.offlineSyncQueue.size
			);
		});
		this.offlineSyncQueue.on("active", () => {
			this.emitRemainingOperationsChange(
				this.fileSyncQueue.size + this.offlineSyncQueue.size
			);
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
		await this.safelySync(async () => {
			try {
				const metadata = this.database.getDocument(relativePath);
				if (metadata) {
					Logger.getInstance().debug(
						`Document metadata already exists for ${relativePath}, it must have been downloaded from the server`
					);
				}

				const contentBytes = await this.operations.read(relativePath);
				const contentHash = hash(contentBytes);

				const response = await this.syncServer.create({
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

				const responseBytes = lib.base64ToBytes(response.contentBase64);
				const responseHash = hash(responseBytes);

				if (contentHash !== responseHash) {
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
					hash: responseHash,
				});

				if (
					this.database.getLastSeenUpdateId() ===
					response.vaultUpdateId - 1
				) {
					await this.database.setLastSeenUpdateId(
						response.vaultUpdateId
					);
				}
			} catch (e) {
				this.history.addHistoryEntry({
					status: SyncStatus.ERROR,
					relativePath,
					message: `Failed to reconcile locally created file: ${e}`,
					type: SyncType.CREATE,
				});
				throw e;
			}
		}, relativePath);
	}

	public async syncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		await this.safelySync(async () => {
			try {
				const metadata = this.database.getDocument(relativePath);
				if (!metadata) {
					this.history.addHistoryEntry({
						status: SyncStatus.NO_OP,
						relativePath,
						message: `Locally deleted file hasn't been uploaded yet, so there's no need to delete it on the remote server`,
						type: SyncType.DELETE,
					});
					return;
				}

				await this.syncServer.delete({
					documentId: metadata.documentId,
					relativePath,
					// We got the event now, so it must have been deleted just now
					createdDate: new Date(),
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					source: SyncSource.PUSH,
					relativePath,
					message: `Successfully deleted locally deleted file on the remote server`,
					type: SyncType.DELETE,
				});

				await this.database.removeDocument(relativePath);
			} catch (e) {
				this.history.addHistoryEntry({
					status: SyncStatus.ERROR,
					relativePath,
					message: `Failed to remotely delete locally deleted file: ${e}`,
					type: SyncType.DELETE,
				});
				throw e;
			}
		}, relativePath);
	}

	public async syncLocallyUpdatedFile({
		oldPath,
		relativePath,
		updateTime,
	}: {
		oldPath?: RelativePath;
		relativePath: RelativePath;
		updateTime: Date;
	}): Promise<void> {
		await this.safelySync(async () => {
			try {
				const metadata = this.database.getDocument(
					oldPath ?? relativePath
				);
				if (!metadata) {
					throw new Error(
						`Document metadata not found for ${relativePath}. Consider resetting the plugin's sync history.`
					);
				}

				const contentBytes = await this.operations.read(relativePath);
				const contentHash = hash(contentBytes);

				if (metadata.hash === contentHash && oldPath !== undefined) {
					this.history.addHistoryEntry({
						status: SyncStatus.NO_OP,
						relativePath,
						message: `File hash matches with last synced version, no need to sync`,
						type: SyncType.UPDATE,
					});
					return;
				}

				const response = await this.syncServer.put({
					documentId: metadata.documentId,
					parentVersionId: metadata.parentVersionId,
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

					if (
						this.database.getLastSeenUpdateId() ===
						response.vaultUpdateId - 1
					) {
						await this.database.setLastSeenUpdateId(
							response.vaultUpdateId
						);
					}

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

				const responseBytes = lib.base64ToBytes(response.contentBase64);
				const responseHash = hash(responseBytes);

				if (response.relativePath != relativePath) {
					await waitForDocumentLock(response.relativePath);

					try {
						await this.operations.move(
							oldPath ?? relativePath,
							response.relativePath
						);
						await this.operations.write(
							response.relativePath,
							contentBytes,
							responseBytes
						);
						this.history.addHistoryEntry({
							status: SyncStatus.SUCCESS,
							source: SyncSource.PULL,
							relativePath,
							message:
								"The file we updated had been moved remotely, therefore, we have moved it locally as well",
							type: SyncType.UPDATE,
						});
					} finally {
						unlockDocument(response.relativePath);
					}
				} else if (contentHash !== responseHash) {
					await this.operations.write(
						relativePath,
						contentBytes,
						responseBytes
					);
				}

				await this.database.moveDocument({
					documentId: metadata.documentId,
					oldRelativePath: oldPath ?? relativePath,
					relativePath: response.relativePath,
					parentVersionId: response.vaultUpdateId,
					hash: responseHash,
				});

				if (
					this.database.getLastSeenUpdateId() ===
					response.vaultUpdateId - 1
				) {
					await this.database.setLastSeenUpdateId(
						response.vaultUpdateId
					);
				}
			} catch (e) {
				this.history.addHistoryEntry({
					status: SyncStatus.ERROR,
					relativePath,
					message: `Failed to reconcile locally updated file: ${e}`,
					type: SyncType.UPDATE,
				});
				throw e;
			}
		}, relativePath);
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
					this.offlineSyncQueue.add(async () => {
						const metadata =
							this.database.getDocument(relativePath);
						if (!metadata) {
							const contentHash = hash(
								await this.operations.read(relativePath)
							);
							const match = locallyDeletedFiles.find(
								([_, document]) => document.hash === contentHash
							);

							if (contentHash != EMPTY_HASH && match) {
								locallyDeletedFiles.remove(match);

								Logger.getInstance().debug(
									`Document ${relativePath} not found in database but found under a different path ${match[0]}, scheduling sync to move it`
								);
								return this.syncLocallyUpdatedFile({
									oldPath: match[0],
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
							return this.syncLocallyCreatedFile(
								relativePath,
								await this.operations.getModificationTime(
									relativePath
								)
							);
						}

						const content = await this.operations.read(
							relativePath
						);
						if (metadata.hash !== hash(content)) {
							Logger.getInstance().debug(
								`Document ${relativePath} has been updated locally, scheduling sync to update it`
							);
							return this.syncLocallyUpdatedFile({
								relativePath: relativePath,
								updateTime:
									await this.operations.getModificationTime(
										relativePath
									),
							});
						}

						this.history.addHistoryEntry({
							status: SyncStatus.NO_OP,
							source: SyncSource.PUSH,
							relativePath,
							message:
								"Document hasn't been updated locally, no need to sync",
						});
						return Promise.resolve();
					})
				)
			);

			await Promise.all(
				locallyDeletedFiles.map(async ([relativePath, _]) => {
					Logger.getInstance().debug(
						`Document ${relativePath} has been deleted locally, scheduling sync to delete it`
					);

					return this.syncLocallyDeletedFile(relativePath);
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

	public async syncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"]
	): Promise<void> {
		await this.safelySync(async () => {
			try {
				const currentVersion = this.database.getDocumentByDocumentId(
					remoteVersion.documentId
				);

				if (!currentVersion) {
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
						await this.syncServer.get({
							documentId: remoteVersion.documentId,
						})
					).contentBase64;
					const contentBytes = lib.base64ToBytes(content);
					const contentHash = hash(contentBytes);

					await this.operations.create(
						remoteVersion.relativePath,
						contentBytes
					);
					await this.database.setDocument({
						documentId: remoteVersion.documentId,
						relativePath: remoteVersion.relativePath,
						parentVersionId: remoteVersion.vaultUpdateId,
						hash: contentHash,
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

				const [relativePath, metadata] = currentVersion;
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
								`Document ${relativePath} has been updated both remotely and locally, skipping until the event is processed`
							);
							return;
						}

						const content = (
							await this.syncServer.get({
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
			} catch (e) {
				this.history.addHistoryEntry({
					status: SyncStatus.ERROR,
					source: SyncSource.PULL,
					relativePath: remoteVersion.relativePath,
					message: `Failed to reconcile remotely updated file: ${e}`,
				});
				throw e;
			}
		}, remoteVersion.relativePath);
	}

	public async reset(): Promise<void> {
		this.fileSyncQueue.clear();
		await this.fileSyncQueue.onEmpty();
		await this.database.resetSyncState();
		this.history.reset();
		this.remainingOperationsListeners.forEach((listener) => {
			listener(0);
		});
	}

	private async safelySync(
		fn: () => Promise<void>,
		relativePath: RelativePath
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
			await this.fileSyncQueue.add(fn);
		} finally {
			unlockDocument(relativePath);
		}
	}

	private emitRemainingOperationsChange(remainingOperations: number): void {
		this.remainingOperationsListeners.forEach((listener) => {
			listener(remainingOperations);
		});
	}
}
