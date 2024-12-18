import { TAbstractFile, TFile } from "obsidian";
import { FileEventHandler } from "./file-event-handler";
import { Logger } from "src/logger";
import { SyncService } from "src/services/sync_service";
import { Database } from "src/database/database";
import { syncLocallyDeletedFile } from "src/sync-operations/sync-locally-deleted-file";
import { syncLocallyUpdatedFile } from "src/sync-operations/sync-locally-updated-file";
import { FileOperations } from "src/file-operations/file-operations";
import { syncLocallyCreatedFile } from "src/sync-operations/sync-locally-created-file";

export class SyncEventHandler implements FileEventHandler {
	public constructor(
		private database: Database,
		private syncServer: SyncService,
		private operations: FileOperations
	) {}

	async onCreate(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File created: ${file.path}`);

			if (!this.database.getSettings().isSyncEnabled) {
				Logger.getInstance().info(
					`Sync is disabled, not syncing ${file.path}`
				);
				return;
			}

			await syncLocallyCreatedFile({
				database: this.database,
				syncServer: this.syncServer,
				operations: this.operations,
				updateTime: new Date(file.stat.ctime),
				filePath: file.path,
			});
		} else {
			Logger.getInstance().info(`Folder created: ${file.path}, ignored`);
		}
	}

	async onDelete(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File deleted: ${file.path}`);

			if (!this.database.getSettings().isSyncEnabled) {
				Logger.getInstance().info(
					`Sync is disabled, not syncing ${file.path}`
				);
				return;
			}

			await syncLocallyDeletedFile({
				database: this.database,
				syncServer: this.syncServer,
				relativePath: file.path,
			});
		} else {
			Logger.getInstance().info(`Folder deleted: ${file.path}, ignored`);
		}
	}

	async onRename(file: TAbstractFile, oldPath: string): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(
				`File renamed: ${oldPath} -> ${file.path}`
			);

			if (!this.database.getSettings().isSyncEnabled) {
				Logger.getInstance().info(
					`Sync is disabled, not syncing ${file.path}`
				);
				return;
			}

			await syncLocallyUpdatedFile({
				database: this.database,
				syncServer: this.syncServer,
				operations: this.operations,
				updateTime: new Date(file.stat.ctime),
				filePath: file.path,
				oldPath,
			});
		} else {
			Logger.getInstance().info(
				`Folder renamed: ${oldPath} -> ${file.path}, ignored`
			);
		}
	}

	async onModify(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File modified: ${file.path}`);

			if (!this.database.getSettings().isSyncEnabled) {
				Logger.getInstance().info(
					`Sync is disabled, not syncing ${file.path}`
				);
				return;
			}

			await syncLocallyUpdatedFile({
				database: this.database,
				syncServer: this.syncServer,
				operations: this.operations,
				updateTime: new Date(file.stat.ctime),
				filePath: file.path,
			});
		} else {
			Logger.getInstance().info(`Folder modified: ${file.path}, ignored`);
		}
	}
}
