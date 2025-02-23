import type { FileSystemOperations } from "dist/types";
import type { RelativePath } from "src/persistence/database";

export class FileNotFoundError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "FileNotFoundError";
	}
}

// Decorate FileSystemOperations replacing errors with FileNotFoundError
// if the accessed file doesn't exist.
export class SafeFileSystemOperations implements FileSystemOperations {
	public constructor(private readonly fs: FileSystemOperations) {}

	public async listAllFiles(): Promise<RelativePath[]> {
		return this.fs.listAllFiles();
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		return this.safeOperation(path, async () => this.fs.read(path));
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		return this.fs.write(path, content);
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: string) => string
	): Promise<string> {
		return this.safeOperation(path, async () =>
			this.fs.atomicUpdateText(path, updater)
		);
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		return this.safeOperation(path, async () => this.fs.getFileSize(path));
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		return this.safeOperation(path, async () =>
			this.fs.getModificationTime(path)
		);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return this.fs.exists(path);
	}

	public async createDirectory(path: RelativePath): Promise<void> {
		return this.fs.createDirectory(path);
	}

	public async delete(path: RelativePath): Promise<void> {
		return this.fs.delete(path);
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		return this.safeOperation(oldPath, async () =>
			this.fs.rename(oldPath, newPath)
		);
	}

	private async safeOperation<T>(
		path: RelativePath,
		operation: () => Promise<T>
	): Promise<T> {
		// Without locking the file, this isn't atomic, however, it's good enough practicaly.
		// This will only break if the file exists, gets deleted and then immediately
		// recreated while `operation` is running.
		if (!(await this.fs.exists(path))) {
			throw new FileNotFoundError(path);
		}
		try {
			return await operation();
		} catch (error) {
			if (await this.fs.exists(path)) {
				throw error;
			} else {
				throw new FileNotFoundError(path);
			}
		}
	}
}
