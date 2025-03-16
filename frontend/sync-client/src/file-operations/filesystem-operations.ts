import type { RelativePath } from "../persistence/database";

export interface FileSystemOperations {
	listAllFiles: () => Promise<RelativePath[]>;
	read: (path: RelativePath) => Promise<Uint8Array>;
	write: (path: RelativePath, content: Uint8Array) => Promise<void>;
	atomicUpdateText: (
		path: RelativePath,
		updater: (currentContent: string) => string
	) => Promise<string>;
	getFileSize: (path: RelativePath) => Promise<number>;
	exists: (path: RelativePath) => Promise<boolean>;
	createDirectory: (path: RelativePath) => Promise<void>;
	delete: (path: RelativePath) => Promise<void>;
	rename: (oldPath: RelativePath, newPath: RelativePath) => Promise<void>;
}
