import { TAbstractFile, TFile } from "obsidian";
import { FileEventHandler } from "./file-event-handler";
import { Logger } from "src/logger";
import { SyncServer } from "src/services/sync_service";
import { Database } from "src/database/database";

export class SyncEventHandler implements FileEventHandler {
	constructor(private database: Database, private syncServer: SyncServer) {}

	async onCreate(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File created: ${file.path}`);

			const result = await this.syncServer.create({
				relativePath: file.path,
				content: await file.vault.readBinary(file),
				createdDate: new Date(file.stat.ctime),
			});

			await this.database.setDocument({
				relativePath: file.path,
				documentId: result.documentId,
				parentVersionId: result.versionId,
			});
		} else {
			Logger.getInstance().info(`Folder created: ${file.path}, ignored`);
		}
	}

	async onDelete(file: TAbstractFile): Promise<void> {
		if (file instanceof TFile) {
			Logger.getInstance().info(`File deleted: ${file.path}`);

			const metadata = this.database.getDocument(file.path);
			if (!metadata) {
				throw `Document metadata not found for ${file.path}`;
			}

			await this.syncServer.delete({
				documentId: metadata.documentId,
				createdDate: new Date(), // We got the event now, so it must have been deleted now
			});

			await this.database.removeDocument(file.path);
		} else {
			Logger.getInstance().info(`Folder deleted: ${file.path}, ignored`);
		}
	}

	async onRename(file: TAbstractFile, oldPath: string): Promise<void> {
		Logger.getInstance().info(`File renamed: ${oldPath} -> ${file.path}`);

		if (file instanceof TFile) {
			const metadata = this.database.getDocument(oldPath);
			if (!metadata) {
				throw `Document metadata not found for ${oldPath}`;
			}

			const response = await this.syncServer.update({
				documentId: metadata.documentId,
				parentVersionId: metadata.parentVersionId,
				relativePath: file.path,
				content: await file.vault.readBinary(file),
				createdDate: new Date(file.stat.ctime),
			});

			await this.database.moveDocument({
				oldRelativePath: oldPath,
				relativePath: file.path,
				documentId: response.documentId,
				parentVersionId: response.versionId,
			});
		} else {
			Logger.getInstance().info(
				`Folder renamed: ${oldPath} -> ${file.path}, ignored`
			);
		}
	}

	async onModify(file: TAbstractFile): Promise<void> {
		Logger.getInstance().info(`File modified: ${file.path}`);

		if (file instanceof TFile) {
			const metadata = this.database.getDocument(file.path);
			if (!metadata) {
				throw `Document metadata not found for ${file.path}`;
			}

			const response = await this.syncServer.update({
				documentId: metadata.documentId,
				parentVersionId: metadata.parentVersionId,
				relativePath: file.path,
				content: await file.vault.readBinary(file),
				createdDate: new Date(file.stat.ctime),
			});

			await this.database.setDocument({
				relativePath: file.path,
				documentId: response.documentId,
				parentVersionId: response.versionId,
			});
		} else {
			Logger.getInstance().info(`Folder modified: ${file.path}, ignored`);
		}
	}
}
