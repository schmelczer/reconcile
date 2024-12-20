import type { TAbstractFile } from "obsidian";
import { TFile } from "obsidian";
import type { FileEventHandler } from "./file-event-handler";
import type { SyncService } from "src/services/sync-service";
import type { Database } from "src/database/database";
import { syncLocallyDeletedFile } from "src/sync-operations/sync-locally-deleted-file";
import { syncLocallyUpdatedFile } from "src/sync-operations/sync-locally-updated-file";
import type { FileOperations } from "src/file-operations/file-operations";
import { syncLocallyCreatedFile } from "src/sync-operations/sync-locally-created-file";
import { Logger } from "src/tracing/logger";
import type { SyncHistory } from "src/tracing/sync-history";

export class ObsidianFileEventHandler implements FileEventHandler {
	public constructor(
		private readonly database: Database,
		private readonly syncServer: SyncService,
		private readonly operations: FileOperations,
		private readonly history: SyncHistory
	) {}

	public async onCreate(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File created: ${file.path}`);

			await syncLocallyCreatedFile({
				database: this.database,
				syncServer: this.syncServer,
				operations: this.operations,
				updateTime: new Date(file.stat.ctime),
				relativePath: file.path,
				history: this.history,
			});
		} else {
			Logger.getInstance().info(`Folder created: ${file.path}, ignored`);
		}
	}

	public async onDelete(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File deleted: ${file.path}`);

			await syncLocallyDeletedFile({
				database: this.database,
				syncServer: this.syncServer,
				history: this.history,
				relativePath: file.path,
			});
		} else {
			Logger.getInstance().info(`Folder deleted: ${file.path}, ignored`);
		}
	}

	public async onRename(file: TAbstractFile, oldPath: string): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(
				`File renamed: ${oldPath} -> ${file.path}`
			);

			await syncLocallyUpdatedFile({
				database: this.database,
				syncServer: this.syncServer,
				operations: this.operations,
				history: this.history,
				updateTime: new Date(file.stat.ctime),
				relativePath: file.path,
				oldPath,
			});
		} else {
			Logger.getInstance().info(
				`Folder renamed: ${oldPath} -> ${file.path}, ignored`
			);
		}
	}

	public async onModify(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File modified: ${file.path}`);

			await syncLocallyUpdatedFile({
				database: this.database,
				syncServer: this.syncServer,
				operations: this.operations,
				history: this.history,
				updateTime: new Date(file.stat.ctime),
				relativePath: file.path,
			});
		} else {
			Logger.getInstance().info(`Folder modified: ${file.path}, ignored`);
		}
	}
}
