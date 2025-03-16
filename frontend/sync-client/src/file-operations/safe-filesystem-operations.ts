import type { RelativePath } from "../persistence/database";
import type { FileSystemOperations } from "./filesystem-operations";
import type { Logger } from "../tracing/logger";
import { DocumentLocks } from "./document-locks";

export class FileNotFoundError extends Error {
	public constructor(message: string) {
		super(message);
		this.name = "FileNotFoundError";
	}
}

// Decorate FileSystemOperations replacing errors with FileNotFoundError
// if the accessed file doesn't exist. It also ensures that there's only
// ever a single request in-flight for any one file through the use of
// DocumentLocks.
export class SafeFileSystemOperations implements FileSystemOperations {
	private readonly locks: DocumentLocks;

	public constructor(
		private readonly fs: FileSystemOperations,
		private readonly logger: Logger
	) {
		this.locks = new DocumentLocks(logger);
	}

	public async listAllFiles(): Promise<RelativePath[]> {
		return this.fs.listAllFiles();
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		this.logger.debug(`Reading file: ${path}`);
		return this.safeOperation(
			path,
			this.decorateToHoldLock(path, async () => this.fs.read(path)),
			"read"
		);
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		this.logger.debug(`Writing file: ${path}`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.write(path, content)
		)();
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: string) => string
	): Promise<string> {
		this.logger.debug(`Atomic update of file: ${path}`);
		return this.safeOperation(
			path,
			this.decorateToHoldLock(path, async () =>
				this.fs.atomicUpdateText(path, updater)
			),
			"atomicUpdateText"
		);
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		this.logger.debug(`Getting file size: ${path}`);
		return this.safeOperation(
			path,
			this.decorateToHoldLock(path, async () =>
				this.fs.getFileSize(path)
			),
			"getFileSize"
		);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		this.logger.debug(`Checking if file exists: ${path}`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.exists(path)
		)();
	}

	public async createDirectory(path: RelativePath): Promise<void> {
		this.logger.debug(`Creating directory: ${path}`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.createDirectory(path)
		)();
	}

	public async delete(path: RelativePath): Promise<void> {
		this.logger.debug(`Deleting file: ${path}`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.delete(path)
		)();
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		this.logger.debug(`Renaming file: ${oldPath} to ${newPath}`);
		return this.safeOperation(
			oldPath,
			this.decorateToHoldLock([oldPath, newPath], async () =>
				this.fs.rename(oldPath, newPath)
			),
			"rename"
		);
	}

	private decorateToHoldLock<T>(
		pathOrPaths: RelativePath | RelativePath[],
		operation: () => Promise<T>
	): () => Promise<T> {
		return async () => {
			const paths = Array.isArray(pathOrPaths)
				? pathOrPaths
				: [pathOrPaths];
			await Promise.all(
				paths.map(async (path) => this.locks.waitForDocumentLock(path))
			);
			try {
				return await operation();
			} finally {
				await Promise.all(
					paths.map((path) => {
						this.locks.unlockDocument(path);
					})
				);
			}
		};
	}

	private async safeOperation<T>(
		path: RelativePath,
		operation: () => Promise<T>,
		operationName: string
	): Promise<T> {
		// Without locking the file, this isn't atomic, however, it's good enough practicaly.
		// This will only break if the file exists, gets deleted and then immediately
		// recreated while `operation` is running.
		if (!(await this.fs.exists(path))) {
			throw new FileNotFoundError(
				`File not found: ${path} before trying to ${operationName}`
			);
		}
		try {
			return await operation();
		} catch (error) {
			if (await this.fs.exists(path)) {
				throw error;
			} else {
				throw new FileNotFoundError(
					`File not found: ${path} when trying to ${operationName}`
				);
			}
		}
	}
}
