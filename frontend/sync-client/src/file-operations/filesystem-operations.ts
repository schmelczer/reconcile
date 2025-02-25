import type { RelativePath } from "src/persistence/database";

export interface FileSystemOperations {
	listAllFiles: () => Promise<RelativePath[]>;
	read: (path: RelativePath) => Promise<Uint8Array>;
	write: (path: RelativePath, content: Uint8Array) => Promise<void>;
	atomicUpdateText: (
		path: RelativePath,
		updater: (currentContent: string) => string
	) => Promise<string>;
	getFileSize: (path: RelativePath) => Promise<number>;
	getModificationTime: (path: RelativePath) => Promise<Date>;
	exists: (path: RelativePath) => Promise<boolean>;
	createDirectory: (path: RelativePath) => Promise<void>;
	delete: (path: RelativePath) => Promise<void>;

	// Must be able to handle renaming to a file that already exists
	rename: (oldPath: RelativePath, newPath: RelativePath) => Promise<void>;
}
