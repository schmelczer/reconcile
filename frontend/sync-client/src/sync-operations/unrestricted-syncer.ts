import type {
	Database,
	DocumentRecord,
	RelativePath
} from "../persistence/database";

import type { SyncService } from "../services/sync-service";
import type { Logger } from "../tracing/logger";
import type { SyncHistory } from "../tracing/sync-history";
import { SyncStatus, SyncType } from "../tracing/sync-history";
import { EMPTY_HASH, hash } from "../utils/hash";
import type { components } from "../services/types";
import { deserialize } from "../utils/deserialize";
import type { Settings } from "../persistence/settings";
import type { FileOperations } from "../file-operations/file-operations";
import { createPromise } from "../utils/create-promise";
import { FileNotFoundError } from "../file-operations/file-not-found-error";
import { SyncResetError } from "../services/sync-reset-error";

export class UnrestrictedSyncer {
	public constructor(
		private readonly logger: Logger,
		private readonly database: Database,
		private readonly settings: Settings,
		private readonly syncService: SyncService,
		private readonly operations: FileOperations,
		private readonly history: SyncHistory
	) {}
	public async unrestrictedSyncLocallyCreatedFile(
		document: DocumentRecord
	): Promise<void> {
		return this.executeSync(
			document.relativePath,
			SyncType.CREATE,
			async () => {
				const contentBytes = await this.operations.read(
					document.relativePath
				); // this can throw FileNotFoundError
				const contentHash = hash(contentBytes);

				const response = await this.syncService.create({
					documentId: document.documentId,
					relativePath: document.relativePath,
					contentBytes
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					relativePath: document.relativePath,
					message: `Successfully uploaded locally created file`,
					type: SyncType.CREATE
				});

				this.database.updateDocumentMetadata(
					{
						parentVersionId: response.vaultUpdateId,
						hash: contentHash
					},
					document
				);

				this.database.addLastSeenUpdateId(response.vaultUpdateId);
			}
		);
	}

	public async unrestrictedSyncLocallyDeletedFile(
		document: DocumentRecord
	): Promise<void> {
		await this.executeSync(
			document.relativePath,
			SyncType.DELETE,
			async () => {
				const response = await this.syncService.delete({
					documentId: document.documentId,
					relativePath: document.relativePath
				});

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					relativePath: document.relativePath,
					message: `Successfully deleted locally deleted file on the remote server`,
					type: SyncType.DELETE
				});

				this.database.updateDocumentMetadata(
					{
						parentVersionId: response.vaultUpdateId,
						hash: EMPTY_HASH
					},
					document
				);

				this.database.addLastSeenUpdateId(response.vaultUpdateId);
			}
		);
	}

	public async unrestrictedSyncLocallyUpdatedFile({
		oldPath,
		document,
		// We use the same code path for both local and remote updates. We need to force the update
		// if there are no local changes but we know that the remote version is newer.
		force = false
	}: {
		oldPath?: RelativePath;
		force?: boolean;
		document: DocumentRecord;
	}): Promise<void> {
		await this.executeSync(
			document.relativePath,
			SyncType.UPDATE,
			async () => {
				const originalRelativePath = document.relativePath;

				if (document.metadata === undefined || document.isDeleted) {
					this.logger.debug(
						`Document ${document.relativePath} has been already deleted, no need to update it`
					);
					return;
				}

				const contentBytes = await this.operations.read(
					document.relativePath
				); // this can throw FileNotFoundError
				let contentHash = hash(contentBytes);

				let response:
					| components["schemas"]["DocumentVersion"]
					| components["schemas"]["DocumentUpdateResponse"]
					| undefined = undefined;
				if (
					document.metadata.hash === contentHash &&
					oldPath === undefined
				) {
					if (!force) {
						this.logger.debug(
							`File hash of ${document.relativePath} matches with last synced version and the path hasn't changed; no need to sync`
						);
						return;
					}

					response = await this.syncService.get({
						documentId: document.documentId
					});
				} else {
					response = await this.syncService.put({
						documentId: document.documentId,
						parentVersionId: document.metadata.parentVersionId,
						relativePath: document.relativePath,
						contentBytes
					});
				}

				// `document` is mutable and reflects the latest state in the local database
				// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
				if (document.isDeleted) {
					this.logger.info(
						`Document ${document.relativePath} has been deleted before we could finish updating it`
					);
					this.database.addLastSeenUpdateId(response.vaultUpdateId);
					return;
				}

				// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
				if (document.metadata === undefined) {
					throw new Error(
						`Document ${document.relativePath} no longer has metadata after updating it, this cannot happen`
					);
				}

				if (
					// `Syncer` creates fake local document metadata for all remote docs with invalid hashes. The parent IDs will likely match
					// the latest versions so we still need to update the local versions to turn the fakes into real metadata.
					document.metadata.parentVersionId > response.vaultUpdateId
				) {
					this.logger.debug(
						`Document ${document.relativePath} is already more up to date than the fetched version`
					);
					this.database.addLastSeenUpdateId(response.vaultUpdateId); // in case the previous `vaultUpdateId` update hasn't made it through
					return;
				}

				if (!force) {
					this.history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						relativePath: document.relativePath,
						message: `Successfully uploaded locally updated file to the remote server`,
						type: SyncType.UPDATE
					});
				}

				if (response.isDeleted) {
					this.history.addHistoryEntry({
						status: SyncStatus.SUCCESS,
						relativePath: document.relativePath,
						message:
							"The file we tried to update had been deleted remotely, therefore, we have deleted it locally",
						type: SyncType.DELETE
					});

					this.database.delete(document.relativePath);
					this.database.updateDocumentMetadata(
						{
							parentVersionId: response.vaultUpdateId,
							hash: EMPTY_HASH
						},
						document
					);

					await this.operations.delete(document.relativePath);

					this.database.addLastSeenUpdateId(response.vaultUpdateId);

					return;
				}

				let actualPath = document.relativePath;

				if (response.relativePath != originalRelativePath) {
					actualPath = response.relativePath;
					await this.operations.move(
						document.relativePath,
						response.relativePath
					); // this can throw FileNotFoundError
				}

				if (
					!("type" in response) ||
					response.type === "MergingUpdate"
				) {
					const responseBytes = deserialize(response.contentBase64);
					contentHash = hash(responseBytes);

					this.database.updateDocumentMetadata(
						{
							parentVersionId: response.vaultUpdateId,
							hash: contentHash
						},
						document
					);

					await this.operations.write(
						actualPath,
						contentBytes,
						responseBytes
					);

					if (!force) {
						this.history.addHistoryEntry({
							status: SyncStatus.SUCCESS,
							relativePath: document.relativePath,
							message: `The file we updated had been updated remotely, so we downloaded the merged version`,
							type: SyncType.UPDATE
						});
					}
				} else {
					this.database.updateDocumentMetadata(
						{
							parentVersionId: response.vaultUpdateId,
							hash: contentHash
						},
						document
					);
				}

				this.database.addLastSeenUpdateId(response.vaultUpdateId);
			}
		);
	}

	public async unrestrictedSyncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"],
		document?: DocumentRecord
	): Promise<void> {
		await this.executeSync(
			remoteVersion.relativePath,
			SyncType.UPDATE,
			async () => {
				if (document?.metadata !== undefined) {
					// If the file exists locally, let's pretend the user has updated it
					// and deal with remote update/deletion within `unrestrictedSyncLocallyUpdatedFile`
					if (
						document.metadata.parentVersionId >=
						remoteVersion.vaultUpdateId
					) {
						this.logger.debug(
							`Document ${remoteVersion.relativePath} is already at least as up to date as the fetched version`
						);

						return;
					}

					return this.unrestrictedSyncLocallyUpdatedFile({
						document,
						force: true
					});
				} else if (remoteVersion.isDeleted) {
					// Either the doc hasn't made it to us before and therefore we don't need to delete it,
					// or we already have it, in which case the preceeding if will deal with it
					this.logger.debug(
						`Document ${remoteVersion.relativePath} has been deleted remotely, no need to sync`
					);
					return;
				}

				const content = (
					await this.syncService.get({
						documentId: remoteVersion.documentId
					})
				).contentBase64;

				document = this.database.getDocumentByDocumentId(
					remoteVersion.documentId
				);

				if (document?.isDeleted === true) {
					this.logger.info(
						`Document ${remoteVersion.relativePath} has been deleted locally before we could finish updating it`
					);
					return;
				}

				if (
					(document?.metadata?.parentVersionId ?? -1) >=
					remoteVersion.vaultUpdateId
				) {
					this.logger.debug(
						`Document ${remoteVersion.relativePath} is already more up to date than the fetched version`
					);
					return;
				}

				const contentBytes = deserialize(content);

				await this.operations.ensureClearPath(
					remoteVersion.relativePath
				);

				const [promise, resolve] = createPromise();
				this.database.updateDocumentMetadata(
					{
						parentVersionId: remoteVersion.vaultUpdateId,
						hash: hash(contentBytes)
					},
					this.database.createNewPendingDocument(
						remoteVersion.documentId,
						remoteVersion.relativePath,
						promise
					)
				);

				await this.operations.create(
					remoteVersion.relativePath,
					contentBytes
				);

				resolve();
				this.database.removeDocumentPromise(promise);

				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					relativePath: remoteVersion.relativePath,
					message: `Successfully downloaded remote file which hadn't existed locally`,
					type: SyncType.CREATE
				});
			}
		);
	}

	public async executeSync<T>(
		relativePath: RelativePath,
		syncType: SyncType,
		fn: () => Promise<T>
	): Promise<T | undefined> {
		this.logger.debug(`Syncing ${relativePath} (${syncType})`);

		try {
			if (await this.operations.exists(relativePath)) {
				const sizeInMB = Math.round(
					(await this.operations.getFileSize(relativePath)) /
						1024 /
						1024
				);

				if (sizeInMB > this.settings.getSettings().maxFileSizeMB) {
					this.history.addHistoryEntry({
						status: SyncStatus.ERROR,
						relativePath,
						message: `File size of ${sizeInMB} MB exceeds the maximum file size limit of ${
							this.settings.getSettings().maxFileSizeMB
						} MB`,
						type: syncType
					});

					return;
				}
			}

			return await fn();
		} catch (e) {
			if (e instanceof FileNotFoundError) {
				// A subsequent sync operation must have been creating to deal with this
				this.logger.info(
					`Skiping file '${relativePath}' because it no longer exists when trying to ${syncType.toLocaleLowerCase()} it`
				);
				return;
			}
			if (e instanceof SyncResetError) {
				this.logger.info(
					`Interrupting sync operation because of a reset`
				);
				return;
			} else {
				this.history.addHistoryEntry({
					status: SyncStatus.ERROR,
					relativePath,
					message: `Failed to sync file '${relativePath}' because of ${e} when trying to ${syncType.toLocaleLowerCase()} it`,
					type: syncType
				});
				throw e;
			}
		}
	}
}
