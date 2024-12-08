import { TAbstractFile, TFile } from "obsidian";
import { FileEventHandler } from "./file-event-handler";
import { Logger } from "src/logger";
import { Syncer } from "src/syncer/syncer";

export class SyncEventHandler implements FileEventHandler {
	constructor(private syncer: Syncer) {}

	async onCreate(file: TAbstractFile) {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File created: ${file}`);
			this.syncer.onCreate(file.path, await file.vault.read(file));
		}
	}

	onDelete(file: TAbstractFile) {
		Logger.getInstance().info(`File deleted: ${file}`);
	}

	onRename(file: TAbstractFile, oldPath: string) {
		Logger.getInstance().info(`File renamed: ${oldPath} -> ${file}`);
	}

	onModify(file: TAbstractFile) {
		Logger.getInstance().info(`File modified: ${file}`);
	}
}
