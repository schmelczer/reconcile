import { TAbstractFile } from "obsidian";

export interface FileEventHandler {
	onCreate: (path: TAbstractFile) => void;
	onDelete: (path: TAbstractFile) => void;
	onRename: (path: TAbstractFile, oldPath: string) => void;
	onModify: (path: TAbstractFile) => void;
}
