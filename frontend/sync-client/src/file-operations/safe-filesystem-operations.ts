import type { RelativePath } from "../persistence/database";
import type { FileSystemOperations } from "./filesystem-operations";
import type { Logger } from "../tracing/logger";
import { Locks } from "../utils/locks";
import { FileNotFoundError } from "./file-not-found-error";

/**
 * Decorates `FileSystemOperations` to replace errors with `FileNotFoundError`
 * if the accessed file doesn't exist. It also ensures that there's at most a
 * single request in-flight for any one file through the use of locks.
 */
export class SafeFileSystemOperations implements FileSystemOperations {
	private readonly locks: Locks<RelativePath>;

	public constructor(
		private readonly fs: FileSystemOperations,
		private readonly logger: Logger
	) {
		this.locks = new Locks(logger);
	}

	public async listAllFiles(): Promise<RelativePath[]> {
		this.logger.debug("Listing all files");
		const result = await this.fs.listAllFiles();
		this.logger.debug(`Listed ${result.length} files`);
		return result;
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		this.logger.debug(`Reading file '${path}'`);
		return this.safeOperation(
			path,
			this.decorateToHoldLock(path, async () => this.fs.read(path)),
			"read"
		);
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		this.logger.debug(`Writing to file '${path}'`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.write(path, content)
		)();
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: string) => string
	): Promise<string> {
		this.logger.debug(`Atomically updating file '${path}'`);
		return this.safeOperation(
			path,
			this.decorateToHoldLock(path, async () =>
				this.fs.atomicUpdateText(path, updater)
			),
			"atomicUpdateText"
		);
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		this.logger.debug(`Getting size of file '${path}'`);
		return this.safeOperation(
			path,
			this.decorateToHoldLock(path, async () =>
				this.fs.getFileSize(path)
			),
			"getFileSize"
		);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		this.logger.debug(`Checking if file '${path}' exists`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.exists(path)
		)();
	}

	public async createDirectory(path: RelativePath): Promise<void> {
		this.logger.debug(`Creating directory '${path}'`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.createDirectory(path)
		)();
	}

	public async delete(path: RelativePath): Promise<void> {
		this.logger.debug(`Deleting file '${path}'`);
		return this.decorateToHoldLock(path, async () =>
			this.fs.delete(path)
		)();
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		this.logger.debug(`Renaming file '${oldPath}' to '${newPath}'`);
		return this.safeOperation(
			oldPath,
			this.decorateToHoldLock([oldPath, newPath], async () =>
				this.fs.rename(oldPath, newPath)
			),
			"rename"
		);
	}

	/**
	 * Decorate an operation to ensure that the file is locked before running it
	 * and that the lock is released afterwards. This results in at-most one
	 * concurrent operation running per file.
	 */
	private decorateToHoldLock<T>(
		pathOrPaths: RelativePath | RelativePath[],
		operation: () => Promise<T>
	): () => Promise<T> {
		return async () => {
			const paths = Array.isArray(pathOrPaths)
				? pathOrPaths
				: [pathOrPaths];

			await Promise.all(
				paths.map(async (path) => this.locks.waitForLock(path))
			);

			try {
				return await operation();
			} finally {
				await Promise.all(
					paths.map((path) => {
						this.locks.unlock(path);
					})
				);
			}
		};
	}

	/**
	 * Decorate an operation to ensure that the file exists before running it.
	 * If the operation fails, it will check if the file still exists and throw
	 * a FileNotFoundError if it doesn't
	 */
	private async safeOperation<T>(
		path: RelativePath,
		operation: () => Promise<T>,
		operationName: string
	): Promise<T> {
		if (!(await this.fs.exists(path))) {
			throw new FileNotFoundError(
				`File '${path}' not found before trying to ${operationName}`
			);
		}

		try {
			return await operation();
		} catch (error) {
			// Without locking the file, this isn't atomic, however, it's good enough in practice.
			// This will only break if the file exists, gets deleted and then immediately
			// recreated while `operation` is running.
			if (await this.fs.exists(path)) {
				throw error;
			} else {
				throw new FileNotFoundError(
					`File '${path}' not found when trying to ${operationName}`
				);
			}
		}
	}
}
