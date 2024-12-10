import { TAbstractFile, TFile } from "obsidian";
import { FileEventHandler } from "./file-event-handler";
import { Logger } from "src/logger";
import { SyncServer } from "src/services/sync_service";
import { Database } from "src/database/database";
import { syncLocallyDeletedFile } from "src/sync-functions/sync-locally-deleted-file";
import { syncLocallyRenamedFile } from "src/sync-functions/sync-locally-renamed-file";
import { syncLocallyUpdatedFile } from "src/sync-functions/sync-locally-updated-file";
import { syncNewLocalFile } from "src/sync-functions/sync-new-local-file";

export class SyncEventHandler implements FileEventHandler {
	constructor(private database: Database, private syncServer: SyncServer) {}

	async onCreate(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File created: ${file.path}`);
			syncNewLocalFile(this.database, this.syncServer, file);
		} else {
			Logger.getInstance().info(`Folder created: ${file.path}, ignored`);
		}
	}

	async onDelete(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File deleted: ${file.path}`);
			syncLocallyDeletedFile(this.database, this.syncServer, file.path);
		} else {
			Logger.getInstance().info(`Folder deleted: ${file.path}, ignored`);
		}
	}

	async onRename(file: TAbstractFile, oldPath: string): Promise<void> {
		Logger.getInstance().info(`File renamed: ${oldPath} -> ${file.path}`);

		if (file instanceof TFile) {
			syncLocallyRenamedFile(
				this.database,
				this.syncServer,
				file,
				oldPath
			);
		} else {
			Logger.getInstance().info(
				`Folder renamed: ${oldPath} -> ${file.path}, ignored`
			);
		}
	}

	async onModify(file: TAbstractFile): Promise<void> {
		Logger.getInstance().info(`File modified: ${file.path}`);

		if (file instanceof TFile) {
			syncLocallyUpdatedFile(this.database, this.syncServer, file);
		} else {
			Logger.getInstance().info(`Folder modified: ${file.path}, ignored`);
		}
	}
}
