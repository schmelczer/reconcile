import type { RelativePath } from "src/database/document-metadata";

export interface FileOperations {
	listAllFiles: () => Promise<RelativePath[]>;

	read: (path: RelativePath) => Promise<Uint8Array>;

	getFileSize(path: RelativePath): Promise<number>;

	getModificationTime: (path: RelativePath) => Promise<Date>;

	// Create and write the file if it doesn't exist. Otherwise, it has the same behavior as write.
	// All parent directories are created if they don't exist.
	create: (path: RelativePath, newContent: Uint8Array) => Promise<void>;

	// Update the file at the given path.
	// If the file's content is different from `expectedContent`, the a 3-way merge is performed before writing.
	// If the file no longer exists, the file is not recreated and an empty array is returned.
	write: (
		path: RelativePath,
		expectedContent: Uint8Array,
		newContent: Uint8Array
	) => Promise<Uint8Array>;

	remove: (path: RelativePath) => Promise<void>;

	move: (oldPath: RelativePath, newPath: RelativePath) => Promise<void>;
}
