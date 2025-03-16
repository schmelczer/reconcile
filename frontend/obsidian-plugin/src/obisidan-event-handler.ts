import type { SyncClient } from "sync-client";
import type { TAbstractFile } from "obsidian";
import { TFile } from "obsidian";

export class ObsidianFileEventHandler {
	public constructor(private readonly client: SyncClient) {}

	public async onCreate(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			this.client.logger.info(`File created: ${file.path}`);

			await this.client.syncer.syncLocallyCreatedFile(file.path);
		} else {
			this.client.logger.debug(`Folder created: ${file.path}, ignored`);
		}
	}

	public async onDelete(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			this.client.logger.info(`File deleted: ${file.path}`);

			await this.client.syncer.syncLocallyDeletedFile(file.path);
		} else {
			this.client.logger.debug(`Folder deleted: ${file.path}, ignored`);
		}
	}

	public async onRename(file: TAbstractFile, oldPath: string): Promise<void> {
		if (file instanceof TFile) {
			this.client.logger.info(`File renamed: ${oldPath} -> ${file.path}`);

			await this.client.syncer.syncLocallyUpdatedFile({
				oldPath,
				relativePath: file.path
			});
		} else {
			this.client.logger.debug(
				`Folder renamed: ${oldPath} -> ${file.path}, ignored`
			);
		}
	}

	public async onModify(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			if (file.basename.startsWith("console-log.iPhone")) {
				return;
			}

			this.client.logger.info(`File modified: ${file.path}`);

			await this.client.syncer.syncLocallyUpdatedFile({
				relativePath: file.path
			});
		} else {
			this.client.logger.debug(`Folder modified: ${file.path}, ignored`);
		}
	}
}
