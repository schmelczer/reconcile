import { TAbstractFile } from "obsidian";

export interface FileEventHandler {
	onCreate: (path: TAbstractFile) => Promise<void>;
	onDelete: (path: TAbstractFile) => Promise<void>;
	onRename: (path: TAbstractFile, oldPath: string) => Promise<void>;
	onModify: (path: TAbstractFile) => Promise<void>;
}
