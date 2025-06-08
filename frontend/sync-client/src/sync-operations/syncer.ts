import type {
	Database,
	DocumentId,
	DocumentRecord,
	RelativePath
} from "../persistence/database";
import type { SyncService } from "../services/sync-service";
import type { Logger } from "../tracing/logger";
import PQueue from "p-queue";
import { hash } from "../utils/hash";
import { v4 as uuidv4 } from "uuid";
import type { Settings, SyncSettings } from "../persistence/settings";
import type { FileOperations } from "../file-operations/file-operations";
import { findMatchingFile } from "../utils/find-matching-file";
import type { UnrestrictedSyncer } from "./unrestricted-syncer";
import { createPromise } from "../utils/create-promise";
import { SyncResetError } from "../services/sync-reset-error";
import { Locks } from "../utils/locks";
import type { DocumentVersionWithoutContent } from "../services/types/DocumentVersionWithoutContent";

export class Syncer {
	private readonly remoteDocumentsLock: Locks<DocumentId>;
	private readonly remainingOperationsListeners: ((
		remainingOperations: number
	) => void)[] = [];
	private readonly syncQueue: PQueue;

	private runningScheduleSyncForOfflineChanges: Promise<void> | undefined;

	// eslint-disable-next-line @typescript-eslint/max-params
	public constructor(
		private readonly deviceId: string,
		private readonly logger: Logger,
		private readonly database: Database,
		private readonly settings: Settings,
		private readonly syncService: SyncService,
		private readonly operations: FileOperations,
		private readonly internalSyncer: UnrestrictedSyncer
	) {
		this.syncQueue = new PQueue({
			concurrency: settings.getSettings().syncConcurrency
		});

		this.remoteDocumentsLock = new Locks<DocumentId>(this.logger);

		settings.addOnSettingsChangeListener((newSettings, oldSettings) => {
			if (newSettings.syncConcurrency !== oldSettings.syncConcurrency) {
				this.syncQueue.concurrency = newSettings.syncConcurrency;
			}
		});

		this.syncQueue.on("active", () => {
			this.remainingOperationsListeners.forEach((listener) => {
				listener(this.syncQueue.size);
			});
		});
	}

	public addRemainingOperationsListener(
		listener: (remainingOperations: number) => void
	): void {
		this.remainingOperationsListeners.push(listener);
	}

	public async syncLocallyCreatedFile(
		relativePath: RelativePath
	): Promise<void> {
		if (
			this.database.getLatestDocumentByRelativePath(relativePath)
				?.isDeleted === false
		) {
			this.logger.debug(
				`Document ${relativePath} already exists in the database, skipping`
			);
			return;
		}

		const [promise, resolve, reject] = createPromise();

		const id = uuidv4();
		const document = this.database.createNewPendingDocument(
			id,
			relativePath,
			promise
		);

		this.logger.debug(
			`Creating new pending document ${relativePath} with id ${id}`
		);

		try {
			await this.syncQueue.add(async () =>
				this.internalSyncer.unrestrictedSyncLocallyCreatedFile(document)
			);

			resolve();
		} catch (e) {
			reject(e);
		} finally {
			this.database.removeDocumentPromise(promise);
		}
	}

	public async syncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		if (
			this.database.getLatestDocumentByRelativePath(relativePath)
				?.isDeleted === true
		) {
			// This is must be a consequence of us deleting a file because of a remote update
			// which triggered a local delete, so we don't need to do anything here.
			this.logger.debug(
				`Document ${relativePath} has already been markes as deleted, skipping`
			);
			return;
		}

		// We have to have a record of the delete in case there's an in-flight update for the same
		// document which finishes after the delete has succeeded and would introduce a phantom metadata record.
		this.database.delete(relativePath);

		const [promise, resolve, reject] = createPromise();

		const document = await this.database.getResolvedDocumentByRelativePath(
			relativePath,
			promise
		);

		try {
			await this.syncQueue.add(async () =>
				this.internalSyncer.unrestrictedSyncLocallyDeletedFile(document)
			);

			resolve();

			this.database.removeDocument(document);
		} catch (e) {
			reject(e);
		} finally {
			this.database.removeDocumentPromise(promise);
		}
	}

	public async syncLocallyUpdatedFile({
		oldPath,
		relativePath
	}: {
		oldPath?: RelativePath;
		relativePath: RelativePath;
	}): Promise<void> {
		if (oldPath !== undefined) {
			// We might have moved the document in the database before calling this method,
			// in that case, we mustn't move it again.
			if (
				this.database.getLatestDocumentByRelativePath(relativePath) ===
					undefined ||
				this.database.getLatestDocumentByRelativePath(relativePath)
					?.isDeleted === true
			) {
				if (oldPath === relativePath) {
					throw new Error(
						`Old path and new path are the same: ${oldPath}`
					);
				}

				this.database.move(oldPath, relativePath);
			}
		}

		let document =
			this.database.getLatestDocumentByRelativePath(relativePath);

		if (
			oldPath !== undefined &&
			document?.metadata?.remoteRelativePath === relativePath
		) {
			this.logger.debug(
				`Document ${relativePath} has been moved as a result of a remote update, skipping sync`
			);
			return;
		}

		if (document === undefined) {
			this.logger.debug(
				`Cannot find document ${relativePath} in the database, skipping`
			);
			return;
		}

		if (document.isDeleted) {
			this.logger.debug(
				`Document ${relativePath} has been deleted locally, skipping`
			);
			return;
		}

		const [promise, resolve, reject] = createPromise();

		document = await this.database.getResolvedDocumentByRelativePath(
			relativePath,
			promise
		);

		try {
			await this.syncQueue.add(async () =>
				this.internalSyncer.unrestrictedSyncLocallyUpdatedFile({
					oldPath,
					document
				})
			);

			resolve();
		} catch (e) {
			reject(e);
		} finally {
			this.database.removeDocumentPromise(promise);
		}
	}

	public async scheduleSyncForOfflineChanges(): Promise<void> {
		if (this.runningScheduleSyncForOfflineChanges !== undefined) {
			this.logger.debug("Uploading local changes is already in progress");
			return this.runningScheduleSyncForOfflineChanges;
		}

		try {
			this.runningScheduleSyncForOfflineChanges =
				this.internalScheduleSyncForOfflineChanges();
			await this.runningScheduleSyncForOfflineChanges;
			this.logger.info(`All local changes have been applied remotely`);
		} catch (e) {
			if (e instanceof SyncResetError) {
				this.logger.info(
					"Failed to apply local changes remotely due to a reset"
				);
				return;
			}
			this.logger.error(
				`Not all local changes have been applied remotely: ${e}`
			);
			throw e;
		} finally {
			this.runningScheduleSyncForOfflineChanges = undefined;
		}
	}

	public async waitUntilFinished(): Promise<void> {
		await this.runningScheduleSyncForOfflineChanges;
		return this.syncQueue.onEmpty();
	}

	public async reset(): Promise<void> {
		await this.waitUntilFinished();
	}

	public async syncRemotelyUpdatedFile(
		remoteVersion: DocumentVersionWithoutContent
	): Promise<void> {
		let document = this.database.getDocumentByDocumentId(
			remoteVersion.documentId
		);

		let hasLockToRelease = false;
		if (document === undefined) {
			// Let's avoid the same documents getting created in parallel multiple times.
			// There might be multiple tasks waiting for the lock
			await this.remoteDocumentsLock.waitForLock(
				remoteVersion.documentId
			);
			hasLockToRelease = true;
			document = this.database.getDocumentByDocumentId(
				remoteVersion.documentId
			);
		}

		try {
			// We're either the first one to get the lock, so we have to create the document in `unrestrictedSyncRemotelyUpdatedFile`
			if (document === undefined) {
				await this.syncQueue.add(async () =>
					this.internalSyncer.unrestrictedSyncRemotelyUpdatedFile(
						remoteVersion
					)
				);
			} else {
				const [promise, resolve, reject] = createPromise();

				document =
					await this.database.getResolvedDocumentByRelativePath(
						document.relativePath,
						promise
					);

				try {
					await this.syncQueue.add(async () =>
						this.internalSyncer.unrestrictedSyncRemotelyUpdatedFile(
							remoteVersion,
							document
						)
					);

					resolve();
				} catch (e) {
					reject(e);
				} finally {
					this.database.removeDocumentPromise(promise);
				}
			}

			this.database.addSeenUpdateId(remoteVersion.vaultUpdateId);
		} finally {
			if (hasLockToRelease) {
				this.remoteDocumentsLock.unlock(remoteVersion.documentId);
			}
		}
	}

	private async internalScheduleSyncForOfflineChanges(): Promise<void> {
		await this.createFakeDocumentsFromRemoteState();

		const allLocalFiles = await this.operations.listAllFiles();

		let locallyPossiblyDeletedFiles: DocumentRecord[] = [];

		for (const document of this.database.resolvedDocuments) {
			if (
				!document.isDeleted &&
				!(await this.operations.exists(document.relativePath))
			) {
				locallyPossiblyDeletedFiles.push(document);
			}
		}

		const updates = Promise.all(
			allLocalFiles.map(async (relativePath) => {
				if (
					this.database.getLatestDocumentByRelativePath(relativePath)
						?.metadata !== undefined
				) {
					this.logger.debug(
						`Document ${relativePath} might have been updated locally, scheduling sync to validate and update it`
					);

					return this.syncLocallyUpdatedFile({
						relativePath
					});
				}

				// Perhaps the file has been moved; let's check by looking at the deleted files
				const contentHash = await this.syncQueue.add(async () => {
					const contentBytes =
						await this.operations.read(relativePath); // this can throw FileNotFoundError
					return hash(contentBytes);
				});

				if (contentHash == undefined) {
					// The file was deleted before we had a chance to read it, no need to sync it here
					return;
				}

				const originalFile = findMatchingFile(
					contentHash,
					locallyPossiblyDeletedFiles
				);
				if (originalFile !== undefined) {
					// `originalFile` hasn't been deleted but it got moved instead
					locallyPossiblyDeletedFiles =
						locallyPossiblyDeletedFiles.filter(
							(item) =>
								item.relativePath !== originalFile.relativePath
						);

					this.logger.debug(
						`Document '${originalFile.relativePath}' was not found under its current path in the database but was found under a different path (${relativePath}), scheduling sync to move it`
					);

					// We're outside of the pqueue, so we need to call the public wrapper
					return this.syncLocallyUpdatedFile({
						oldPath: originalFile.relativePath,
						relativePath
					});
				}

				this.logger.debug(
					`Document ${relativePath} not found in database, scheduling sync to create it`
				);
				// We're outside of the pqueue, so we need to call the public wrapper
				return this.syncLocallyCreatedFile(relativePath);
			})
		);

		const deletes = Promise.all(
			locallyPossiblyDeletedFiles.map(async ({ relativePath }) => {
				this.logger.debug(
					`Document ${relativePath} has been deleted locally, scheduling sync to delete it`
				);

				// We're outside of the pqueue, so we need to call the public wrapper
				return this.syncLocallyDeletedFile(relativePath);
			})
		);

		await Promise.all([updates, deletes]);
	}

	/**
	 * Create fake documents in the database for all files that are present locally
	 * and also exist remotely. This will stop the subequent syncs from duplicating
	 * the documents by creating the same documents from multiple clients.
	 */
	private async createFakeDocumentsFromRemoteState(): Promise<void> {
		if (this.database.getHasInitialSyncCompleted()) {
			return;
		}

		const [allLocalFiles, remote] = await Promise.all([
			this.operations.listAllFiles(),
			this.syncQueue.add(async () => this.syncService.getAll())
		]);

		if (remote !== undefined) {
			remote.latestDocuments
				.filter(
					(remoteDocument) =>
						allLocalFiles.includes(remoteDocument.relativePath) &&
						!remoteDocument.isDeleted &&
						this.database.getDocumentByDocumentId(
							remoteDocument.documentId
						) === undefined
				)
				.forEach((remoteDocument) => {
					this.database.createNewEmptyDocument(
						remoteDocument.documentId,
						remoteDocument.vaultUpdateId,
						remoteDocument.relativePath
					);
				});
		}

		this.database.setHasInitialSyncCompleted(true);
	}
}
