import { TAbstractFile } from "obsidian";
import { FileEventHandler } from "./file-event-handler";
import { Logger } from "src/logger";

export class SyncEventHandler implements FileEventHandler {
	onCreate(path: TAbstractFile) {
		Logger.getInstance().info(`File created: ${path}`);
	}

	onDelete(path: TAbstractFile) {
		Logger.getInstance().info(`File deleted: ${path}`);
	}

	onRename(path: TAbstractFile, oldPath: string) {
		Logger.getInstance().info(`File renamed: ${oldPath} -> ${path}`);
	}

	onModify(path: TAbstractFile) {
		Logger.getInstance().info(`File modified: ${path}`);
	}
}
