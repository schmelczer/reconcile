import { RelativePath } from "src/database/document-metadata";

export interface FileOperations {
	read(path: RelativePath): Promise<Uint8Array>;

	create(path: RelativePath, newContent: Uint8Array): Promise<void>;

	// Writes new content to the file at the given path. If the file's content has changed since the expectedContent was read, the write will merge the changes.
	write(
		path: RelativePath,
		expectedContent: Uint8Array,
		newContent: Uint8Array
	): Promise<Uint8Array>;

	remove(path: RelativePath): Promise<void>;

	move(oldPath: RelativePath, newPath: RelativePath): Promise<void>;
}
