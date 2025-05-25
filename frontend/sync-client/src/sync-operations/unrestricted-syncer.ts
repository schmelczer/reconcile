import type {
	Database,
	DocumentRecord,
	RelativePath
} from "../persistence/database";

import type { SyncService } from "../services/sync-service";
import type { Logger } from "../tracing/logger";
import type {
	CommonHistoryEntry,
	SyncCreateDetails,
	SyncDeleteDetails,
	SyncDetails,
	SyncHistory,
	SyncMovedDetails,
	SyncUpdateDetails
} from "../tracing/sync-history";
import { SyncStatus, SyncType } from "../tracing/sync-history";
import { EMPTY_HASH, hash } from "../utils/hash";
import type { components } from "../services/types";
import { deserialize } from "../utils/deserialize";
import type { Settings } from "../persistence/settings";
import type { FileOperations } from "../file-operations/file-operations";
import { createPromise } from "../utils/create-promise";
import { FileNotFoundError } from "../file-operations/file-not-found-error";
import { SyncResetError } from "../services/sync-reset-error";
import { globsToRegexes } from "../utils/globs-to-regexes";

export class UnrestrictedSyncer {
	private ignorePatterns: RegExp[];

	public constructor(
		private readonly logger: Logger,
		private readonly database: Database,
		private readonly settings: Settings,
		private readonly syncService: SyncService,
		private readonly operations: FileOperations,
		private readonly history: SyncHistory
	) {
		this.ignorePatterns = globsToRegexes(
			this.settings.getSettings().ignorePatterns,
			this.logger
		);

		this.settings.addOnSettingsChangeListener((newSettings) => {
			this.ignorePatterns = globsToRegexes(
				newSettings.ignorePatterns,
				this.logger
			);
		});
	}

	public async unrestrictedSyncLocallyCreatedFile(
		document: DocumentRecord
	): Promise<void> {
		const updateDetails: SyncCreateDetails = {
			type: SyncType.CREATE,
			relativePath: document.relativePath
		};

		return this.executeSync(updateDetails, async () => {
			if (document.isDeleted) {
				this.logger.debug(
					`Document ${document.relativePath} has been already deleted, no need to create it`
				);
				return;
			}

			const contentBytes = await this.operations.read(
				document.relativePath
			); // this can throw FileNotFoundError
			const contentHash = hash(contentBytes);

			const response = await this.syncService.create({
				documentId: document.documentId,
				relativePath: document.relativePath,
				contentBytes
			});

			this.database.updateDocumentMetadata(
				{
					parentVersionId: response.vaultUpdateId,
					hash: contentHash,
					remoteRelativePath: response.relativePath
				},
				document
			);

			this.database.addSeenUpdateId(response.vaultUpdateId);

			this.history.addHistoryEntry({
				status: SyncStatus.SUCCESS,
				details: updateDetails,
				message: `Successfully uploaded locally created file`
			});
		});
	}

	public async unrestrictedSyncLocallyDeletedFile(
		document: DocumentRecord
	): Promise<void> {
		const updateDetails: SyncDeleteDetails = {
			type: SyncType.DELETE,
			relativePath: document.relativePath
		};

		await this.executeSync(updateDetails, async () => {
			const response = await this.syncService.delete({
				documentId: document.documentId,
				relativePath: document.relativePath
			});

			this.database.updateDocumentMetadata(
				{
					parentVersionId: response.vaultUpdateId,
					hash: EMPTY_HASH,
					remoteRelativePath: document.relativePath
				},
				document
			);

			this.database.addSeenUpdateId(response.vaultUpdateId);

			this.history.addHistoryEntry({
				status: SyncStatus.SUCCESS,
				details: updateDetails,
				message: `Successfully deleted locally deleted file on the server`,
				author: response.userId
			});
		});
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
		const updateDetails: SyncUpdateDetails | SyncMovedDetails =
			oldPath !== undefined
				? {
						type: SyncType.MOVE,
						relativePath: document.relativePath,
						movedFrom: oldPath
					}
				: {
						type: SyncType.UPDATE,
						relativePath: document.relativePath
					};

		await this.executeSync(updateDetails, async () => {
			const originalRelativePath = document.relativePath;

			if (document.isDeleted || document.metadata === undefined) {
				this.logger.debug(
					`Document ${document.relativePath} has been already deleted, no need to update it`
				);
				return;
			}

			const contentBytes = await this.operations.read(
				document.relativePath
			); // this can throw FileNotFoundError
			let contentHash = hash(contentBytes);

			const areThereLocalChanges = !(
				document.metadata.hash === contentHash && oldPath === undefined
			);

			let response:
				| components["schemas"]["DocumentVersion"]
				| components["schemas"]["DocumentUpdateResponse"]
				| undefined = undefined;

			if (areThereLocalChanges) {
				response = await this.syncService.put({
					documentId: document.documentId,
					parentVersionId: document.metadata.parentVersionId,
					relativePath: document.relativePath,
					contentBytes
				});
			} else {
				if (!force) {
					this.logger.debug(
						`File hash of ${document.relativePath} matches with last synced version and the path hasn't changed; no need to sync`
					);
					return;
				}

				response = await this.syncService.get({
					documentId: document.documentId
				});
			}

			// `document` is mutable and reflects the latest state in the local database
			// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
			if (document.isDeleted) {
				this.logger.info(
					`Document ${document.relativePath} has been deleted before we could finish updating it`
				);
				this.database.addSeenUpdateId(response.vaultUpdateId);
				return;
			}

			if (
				// `Syncer` creates fake local document metadata for all remote docs with invalid hashes. The parent IDs will likely match
				// the latest versions so we still need to update the local versions to turn the fakes into real metadata.
				document.metadata.parentVersionId > response.vaultUpdateId
			) {
				this.logger.debug(
					`Document ${document.relativePath} is already more up to date than the fetched version`
				);
				this.database.addSeenUpdateId(response.vaultUpdateId); // in case the previous `vaultUpdateId` update hasn't made it through
				return;
			}

			if (response.isDeleted) {
				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					details: {
						type: SyncType.DELETE,
						relativePath: document.relativePath
					},
					message:
						"File has been deleted remotely, so we deleted it locally",
					author: response.userId
				});

				this.database.delete(document.relativePath);
				this.database.updateDocumentMetadata(
					{
						parentVersionId: response.vaultUpdateId,
						hash: EMPTY_HASH,
						remoteRelativePath: response.relativePath
					},
					document
				);

				await this.operations.delete(document.relativePath);

				this.database.addSeenUpdateId(response.vaultUpdateId);

				return;
			}

			let actualPath = document.relativePath;

			if (response.relativePath != originalRelativePath) {
				actualPath = response.relativePath;
				// Make sure to update the remote relative path to avoid uploading
				// the file as a result of this filesystem event.
				document.metadata.remoteRelativePath = response.relativePath;
				await this.operations.move(
					document.relativePath,
					response.relativePath
				); // this can throw FileNotFoundError
			}

			if (!("type" in response) || response.type === "MergingUpdate") {
				const responseBytes = deserialize(response.contentBase64);
				contentHash = hash(responseBytes);

				this.database.updateDocumentMetadata(
					{
						parentVersionId: response.vaultUpdateId,
						hash: contentHash,
						remoteRelativePath: response.relativePath
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
						details: updateDetails,
						message: `The file we updated had been updated remotely, so we downloaded the merged version`
					});
				}
			} else {
				this.database.updateDocumentMetadata(
					{
						parentVersionId: response.vaultUpdateId,
						hash: contentHash,
						remoteRelativePath: response.relativePath
					},
					document
				);
			}

			this.database.addSeenUpdateId(response.vaultUpdateId);

			const actualUpdateDetails: SyncUpdateDetails | SyncMovedDetails =
				oldPath !== undefined ||
				response.relativePath != originalRelativePath
					? {
							type: SyncType.MOVE,
							relativePath: response.relativePath,
							movedFrom: originalRelativePath
						}
					: {
							type: SyncType.UPDATE,
							relativePath: response.relativePath
						};

			if (areThereLocalChanges) {
				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					details: actualUpdateDetails,
					message: `Successfully uploaded locally updated file to the server`,
					author: response.userId
				});
			} else {
				this.history.addHistoryEntry({
					status: SyncStatus.SUCCESS,
					details: actualUpdateDetails,
					message: `Successfully downloaded remotely updated file from the server`,
					author: response.userId
				});
			}
		});
	}

	public async unrestrictedSyncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"],
		document?: DocumentRecord
	): Promise<void> {
		const updateDetails: SyncCreateDetails = {
			type: SyncType.CREATE,
			relativePath: remoteVersion.relativePath
		};

		await this.executeSync(updateDetails, async () => {
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
				// Either the document hasn't made it to us before and therefore we don't need to delete it,
				// or we already have it, in which case the preceeding if would've dealt with it
				this.logger.debug(
					`Document ${remoteVersion.relativePath} has been deleted remotely, no need to sync`
				);
				return;
			}

			// Don't download oversized files
			const historyEntryForSkippedOversizedFile =
				this.getHistoryEntryForSkippedOversizedFile(
					remoteVersion.contentSize,
					remoteVersion.relativePath
				);
			if (historyEntryForSkippedOversizedFile !== undefined) {
				this.history.addHistoryEntry(
					historyEntryForSkippedOversizedFile
				);
				return;
			}

			const content = (
				await this.syncService.get({
					documentId: remoteVersion.documentId
				})
			).contentBase64;

			// We're trying to create an entirely new document that didn't exist locally
			document = this.database.getDocumentByDocumentId(
				remoteVersion.documentId
			);
			// It can happen that a concurrent sync operation has already created the document, so we can bail here
			if (document !== undefined) {
				this.logger.debug(
					`Document ${remoteVersion.relativePath} has already been created locally, no need to create it again`
				);
				return;
			}

			const contentBytes = deserialize(content);

			await this.operations.ensureClearPath(remoteVersion.relativePath);

			const [promise, resolve] = createPromise();
			this.database.updateDocumentMetadata(
				{
					parentVersionId: remoteVersion.vaultUpdateId,
					hash: hash(contentBytes),
					remoteRelativePath: remoteVersion.relativePath
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
				details: updateDetails,
				message: `Successfully downloaded remote file which hadn't existed locally`,
				author: remoteVersion.userId
			});
		});
	}

	public async executeSync<T>(
		details: SyncDetails,
		fn: () => Promise<T>
	): Promise<T | undefined> {
		for (const pattern of this.ignorePatterns) {
			if (pattern.test(details.relativePath)) {
				this.logger.debug(
					`File '${details.relativePath}' is ignored by the ignore pattern: ${pattern}`
				);
				return; // bail without SKIPPED status because we were told to ignore this file and we shouldn't clutter up the history
			}
		}

		try {
			// Only check the size of files which already exist locally.
			if (await this.operations.exists(details.relativePath)) {
				const sizeInBytes = await this.operations.getFileSize(
					details.relativePath
				);
				const historyEntryForSkippedOversizedFile =
					this.getHistoryEntryForSkippedOversizedFile(
						sizeInBytes,
						details.relativePath
					);
				if (historyEntryForSkippedOversizedFile !== undefined) {
					this.history.addHistoryEntry(
						historyEntryForSkippedOversizedFile
					);
					return;
				}
			}

			return await fn();
		} catch (e) {
			if (e instanceof FileNotFoundError) {
				// A subsequent sync operation must have been creating to deal with this
				this.logger.info(
					`Skiping file '${details.relativePath}' because it no longer exists when trying to ${details.type.toLocaleLowerCase()} it`
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
					details,
					message: `Failed to sync file '${details.relativePath}' because of ${e} when trying to ${details.type.toLocaleLowerCase()} it`
				});
				throw e;
			}
		}
	}

	private getHistoryEntryForSkippedOversizedFile(
		sizeInBytes: number,
		relativePath: RelativePath
	): CommonHistoryEntry | undefined {
		const sizeInMB = Math.round(sizeInBytes / 1024 / 1024);
		const { maxFileSizeMB } = this.settings.getSettings();
		if (sizeInMB > maxFileSizeMB) {
			return {
				status: SyncStatus.SKIPPED,
				details: {
					type: SyncType.SKIPPED,
					relativePath
				},
				message: `File size of ${sizeInMB} MB exceeds the maximum file size limit of ${
					maxFileSizeMB
				} MB`
			};
		}
	}
}
