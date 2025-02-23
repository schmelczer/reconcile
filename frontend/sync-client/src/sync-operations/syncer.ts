import type { Database, RelativePath } from "../persistence/database";

import type { SyncService } from "src/services/sync-service";
import type { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";
import { SyncSource, SyncStatus, SyncType } from "src/tracing/sync-history";
import { unlockDocument, waitForDocumentLock } from "./document-lock";
import PQueue from "p-queue";
import { hash } from "src/utils/hash";
import type { components } from "src/services/types";
import { deserialize } from "src/utils/deserialize";
import type { Settings } from "src/persistence/settings";
import type { FileOperations } from "src/file-operations/file-operations";
import { findMatchingFileBasedOnHash } from "src/utils/find-matching-file-based-on-hash";
import { UnrestrictedSyncer } from "./unrestricted-syncer";

export class Syncer {
	private readonly remainingOperationsListeners: ((
		remainingOperations: number
	) => void)[] = [];

	private readonly syncQueue: PQueue;

	private runningScheduleSyncForOfflineChanges: Promise<void> | undefined =
		undefined;
	private runningApplyRemoteChangesLocally: Promise<void> | undefined =
		undefined;

	private readonly internalSyncer: UnrestrictedSyncer;

	public constructor(
		private readonly logger: Logger,
		private readonly database: Database,
		private readonly settings: Settings,
		private readonly syncService: SyncService,
		private readonly operations: FileOperations,
		history: SyncHistory
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

		this.internalSyncer = new UnrestrictedSyncer(
			logger,
			database,
			settings,
			syncService,
			operations,
			history
		);
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
			this.internalSyncer.unrestrictedSyncLocallyCreatedFile(
				relativePath,
				updateTime
			)
		);
	}

	public async syncLocallyUpdatedFile(args: {
		oldPath?: RelativePath;
		relativePath: RelativePath;
		updateTime: Date;
	}): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncer.unrestrictedSyncLocallyUpdatedFile(args)
		);
	}

	public async waitForSyncQueue(): Promise<void> {
		return this.syncQueue.onEmpty();
	}

	public async syncLocallyDeletedFile(
		relativePath: RelativePath
	): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncer.unrestrictedSyncLocallyDeletedFile(relativePath)
		);
	}

	private async syncRemotelyUpdatedFile(
		remoteVersion: components["schemas"]["DocumentVersionWithoutContent"]
	): Promise<void> {
		await this.syncQueue.add(async () =>
			this.internalSyncer.unrestrictedSyncRemotelyUpdatedFile(
				remoteVersion
			)
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

		// This includes renamed files for now
		let locallyPossiblyDeletedFiles = [
			...this.database.getDocuments().entries()
		].filter(([path, _]) => !allLocalFiles.includes(path));

		await Promise.all(
			allLocalFiles.map(async (relativePath) =>
				this.syncQueue.add(async () => {
					const metadata = this.database.getDocument(relativePath);

					if (metadata) {
						this.logger.debug(
							`Document ${relativePath} has been updated locally, scheduling sync to update it`
						);
						return this.internalSyncer.unrestrictedSyncLocallyUpdatedFile(
							{
								relativePath,
								updateTime:
									await this.operations.getModificationTime(
										relativePath
									)
							}
						);
					}

					// Perhaps the file has been moved. Let's check by looking at the deleted files
					const contentBytes =
						await this.operations.read(relativePath);
					const contentHash = hash(contentBytes);

					const originalFile = findMatchingFileBasedOnHash(
						contentHash,
						locallyPossiblyDeletedFiles
					);
					if (originalFile !== undefined) {
						// `originalFile` hasn't been deleted but it got moved instead
						locallyPossiblyDeletedFiles =
							locallyPossiblyDeletedFiles.filter(
								(item) => item[0] !== originalFile[0]
							);

						this.logger.debug(
							`Document ${relativePath} was not found under its current path in the database but was found under a different path ${originalFile[0]}, scheduling sync to move it`
						);
						return this.internalSyncer.unrestrictedSyncLocallyUpdatedFile(
							{
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
							}
						);
					}

					this.logger.debug(
						`Document ${relativePath} not found in database, scheduling sync to create it`
					);
					return this.internalSyncer.unrestrictedSyncLocallyCreatedFile(
						relativePath,
						await this.operations.getModificationTime(relativePath)
					);
				})
			)
		);

		await Promise.all(
			locallyPossiblyDeletedFiles.map(async ([relativePath, _]) => {
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

	private emitRemainingOperationsChange(remainingOperations: number): void {
		this.remainingOperationsListeners.forEach((listener) => {
			listener(remainingOperations);
		});
	}
}
