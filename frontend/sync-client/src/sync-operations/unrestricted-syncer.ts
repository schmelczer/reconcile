import type { Database, RelativePath } from "../persistence/database";

import type { SyncService } from "src/services/sync-service";
import type { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history";
import { hash } from "src/utils/hash";
import type { components } from "src/services/types";
import { deserialize } from "src/utils/deserialize";
import type { Settings } from "src/persistence/settings";
import type { FileOperations } from "src/file-operations/file-operations";
import { FileNotFoundError } from "src/file-operations/safe-filesystem-operations";
import { DocumentLocks } from "./document-locks";

export class UnrestrictedSyncer {
	private readonly locks = new DocumentLocks();

	public constructor(
		private readonly logger: Logger,
		private readonly database: Database,
		private readonly settings: Settings,
		private readonly syncService: SyncService,
		private readonly operations: FileOperations,
		private readonly history: SyncHistory
	) {}

	public async unrestrictedSyncLocallyCreatedFile(
		relativePath: RelativePath,
		updateTime: Date,
		optimisations?: {
			contentBytes?: Uint8Array;
			contentHash?: string;
		}
	): Promise<void> {
		await this.executeWhileHoldingFileLock(
			[relativePath],
			SyncType.CREATE,
			SyncSource.PUSH,
			async () => {
				if (
					(await this.operations.getFileSize(relativePath)) / // this can throw FileNotFoundError
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
					(await this.operations.read(relativePath)); // this can throw FileNotFoundError
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

				// The response can't have a different relative path than the one we sent
				// because the relative path is the key when finding existing documents
				// when a create request is sent.

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

	public async unrestrictedSyncLocallyUpdatedFile({
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
			[oldPath, relativePath].filter((path) => path !== undefined),
			SyncType.UPDATE,
			SyncSource.PUSH,
			async () => {
				const localMetadata = this.database.getDocument(
					oldPath ?? relativePath
				);

				if (!localMetadata) {
					this.history.addHistoryEntry({
						status: SyncStatus.NO_OP,
						relativePath,
						message: `Document metadata doesn't exist for ${oldPath ?? relativePath}, it must have been already deleted`,
						type: SyncType.UPDATE
					});
					return;
				}

				if (
					(await this.operations.getFileSize(relativePath)) / // this can throw FileNotFoundError
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
					(await this.operations.read(relativePath)); // this can throw FileNotFoundError

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

				if (
					response.relativePath != relativePath &&
					response.relativePath != oldPath
				) {
					await this.locks.waitForDocumentLock(response.relativePath);
				}

				try {
					if (response.relativePath != relativePath) {
						// TODO: this can fail, that's bad
						await this.operations.move(
							// this can throw FileNotFoundError
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
					if (
						response.relativePath != relativePath &&
						response.relativePath != oldPath
					) {
						this.locks.unlockDocument(response.relativePath);
					}
				}
			}
		);
	}

	public async unrestrictedSyncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		await this.executeWhileHoldingFileLock(
			[relativePath],
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

	public async unrestrictedSyncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"]
	): Promise<void> {
		await this.executeWhileHoldingFileLock(
			[remoteVersion.relativePath],
			SyncType.UPDATE,
			SyncSource.PULL,
			async () => {
				let localMetadata = this.database.getDocumentByDocumentId(
					remoteVersion.documentId
				);

				if (
					localMetadata &&
					localMetadata[0] !== remoteVersion.relativePath
				) {
					await this.locks.waitForDocumentLock(localMetadata[0]);
				}
				// Waiting for the new lock might take a while so we need to fetch the database
				// entry again in case it's changed.
				localMetadata = this.database.getDocumentByDocumentId(
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
						// TODO: this can fail, that's bad
						const currentContent =
							await this.operations.read(relativePath); // this can throw FileNotFoundError
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
							// TODO: this can fail, that's bad
							await this.operations.move(
								// this can throw FileNotFoundError
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
						this.locks.unlockDocument(relativePath);
					}
				}
			}
		);
	}

	public async executeWhileHoldingFileLock(
		lockedPaths: RelativePath[],
		syncType: SyncType,
		syncSource: SyncSource,
		fn: () => Promise<void>
	): Promise<void> {
		const relativePath = lockedPaths[lockedPaths.length - 1];

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

		await Promise.all(
			lockedPaths.map(this.locks.waitForDocumentLock.bind(this.locks))
		);
		try {
			await fn();
		} catch (e) {
			if (e instanceof FileNotFoundError) {
				// A subsequent sync operation must have been creating to deal with this
				this.history.addHistoryEntry({
					status: SyncStatus.NO_OP,
					relativePath,
					message: `Skip ${syncSource.toLocaleLowerCase()} file because it no longer exists when trying to ${syncType.toLocaleLowerCase()} it`,
					type: syncType,
					source: syncSource
				});
			} else {
				this.history.addHistoryEntry({
					status: SyncStatus.ERROR,
					relativePath,
					message: `Failed to ${syncSource.toLocaleLowerCase()} file because of ${e} when trying to ${syncType.toLocaleLowerCase()} it`,
					type: syncType,
					source: syncSource
				});
				throw e;
			}
		} finally {
			lockedPaths.forEach(this.locks.unlockDocument.bind(this.locks));
		}
	}

	public reset(): void {
		this.locks.reset();
	}

	private async tryIncrementVaultUpdateId(
		responseVaultUpdateId: number
	): Promise<void> {
		if (this.database.getLastSeenUpdateId() === responseVaultUpdateId - 1) {
			await this.database.setLastSeenUpdateId(responseVaultUpdateId);
		}
	}
}
