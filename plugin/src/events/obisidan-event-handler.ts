import type { TAbstractFile } from "obsidian";
import { TFile } from "obsidian";
import type { FileEventHandler } from "./file-event-handler";
import { Logger } from "src/tracing/logger";
import type { Syncer } from "src/sync-operations/syncer";

export class ObsidianFileEventHandler implements FileEventHandler {
	public constructor(private readonly syncer: Syncer) {}

	public async onCreate(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File created: ${file.path}`);

			await this.syncer.syncLocallyCreatedFile(
				file.path,
				new Date(file.stat.ctime)
			);
		} else {
			Logger.getInstance().debug(`Folder created: ${file.path}, ignored`);
		}
	}

	public async onDelete(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File deleted: ${file.path}`);

			await this.syncer.syncLocallyDeletedFile(file.path);
		} else {
			Logger.getInstance().debug(`Folder deleted: ${file.path}, ignored`);
		}
	}

	public async onRename(file: TAbstractFile, oldPath: string): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(
				`File renamed: ${oldPath} -> ${file.path}`
			);

			await this.syncer.syncLocallyUpdatedFile({
				oldPath,
				relativePath: file.path,
				updateTime: new Date(file.stat.ctime),
			});
		} else {
			Logger.getInstance().debug(
				`Folder renamed: ${oldPath} -> ${file.path}, ignored`
			);
		}
	}

	public async onModify(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File modified: ${file.path}`);

			await this.syncer.syncLocallyUpdatedFile({
				relativePath: file.path,
				updateTime: new Date(file.stat.ctime),
			});
		} else {
			Logger.getInstance().debug(
				`Folder modified: ${file.path}, ignored`
			);
		}
	}
}
