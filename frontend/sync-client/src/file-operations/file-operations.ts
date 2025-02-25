import type { Logger } from "src/tracing/logger";
import type { FileSystemOperations } from "./filesystem-operations";
import type {
	Database,
	DocumentId,
	RelativePath
} from "src/persistence/database";
import { isBinary, isFileTypeMergable, mergeText } from "sync_lib";
import { SafeFileSystemOperations } from "./safe-filesystem-operations";

export class FileOperations {
	private static readonly PARENTHESES_REGEX = / \((\d+)\)$/;
	private readonly fs: SafeFileSystemOperations;

	public constructor(
		private readonly logger: Logger,
		private readonly database: Database,
		fs: FileSystemOperations
	) {
		this.fs = new SafeFileSystemOperations(fs);
	}

	public async listAllFiles(): Promise<RelativePath[]> {
		const files = await this.fs.listAllFiles();
		this.logger.debug(`Listing all files, found ${files.length}`);
		return files;
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		const content = await this.fs.read(path);

		if (isBinary(content)) {
			return content;
		}

		const decoder = new TextDecoder("utf-8");

		// Normalize line endings to LF on Windows
		let text = decoder.decode(content);
		text = text.replace(/\r\n/g, "\n");

		return new TextEncoder().encode(text);
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		return this.fs.getFileSize(path);
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		return this.fs.getModificationTime(path);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return this.fs.exists(path);
	}

	// Create and write the file if it doesn't exist. Otherwise, it has the same behavior as write.
	// All parent directories are created if they don't exist.
	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		if (await this.fs.exists(path)) {
			const deconflictedPath = await this.deconflictPath(path);
			this.logger.debug(
				`Didn't expect ${path} to exist, deconflicting by moving it to '${deconflictedPath}'`
			);
			await this.database.updatePath(path, deconflictedPath);
			await this.fs.rename(path, deconflictedPath);
		} else {
			await this.createParentDirectories(path);
		}

		await this.fs.write(path, newContent);
	}

	// Update the file at the given path.
	// If the file's content is different from `expectedContent`, the a 3-way merge is performed before writing.
	// If the file no longer exists, the file is not recreated and an empty array is returned.
	public async write(
		path: RelativePath,
		expectedContent: Uint8Array,
		newContent: Uint8Array
	): Promise<Uint8Array> {
		if (!(await this.fs.exists(path))) {
			this.logger.debug(
				`The caller assumed ${path} exists, but it no longer, so we wont recreate it`
			);
			return new Uint8Array(0);
		}

		if (
			!isFileTypeMergable(path) ||
			isBinary(expectedContent) ||
			isBinary(newContent)
		) {
			this.logger.debug(
				`The expected content is not mergable, so we won't perform a 3-way merge, just overwrite it`
			);
			await this.fs.write(path, newContent);
			return newContent;
		}

		const expectedText = new TextDecoder().decode(expectedContent);
		const newText = new TextDecoder().decode(newContent);

		const resultText = await this.fs.atomicUpdateText(
			path,
			(currentText) => {
				currentText = currentText.replace(/\r\n/g, "\n");
				if (currentText !== expectedText) {
					this.logger.debug(
						`Performing a 3-way merge for ${path} with the expected content`
					);

					return mergeText(expectedText, currentText, newText);
				}

				this.logger.debug(
					`The current content of ${path} is the same as the expected content, so we will just write the new content`
				);

				return newText;
			}
		);
		return new TextEncoder().encode(resultText);
	}

	public async remove(path: RelativePath): Promise<void> {
		this.logger.debug(`Deleting file: ${path}`);
		return this.fs.delete(path);
	}

	public async move(
		oldPath: RelativePath,
		newPath: RelativePath,
		documentId?: DocumentId
	): Promise<void> {
		if (oldPath === newPath) {
			return;
		}

		if (await this.fs.exists(newPath)) {
			const deconflictedPath = await this.deconflictPath(newPath);
			this.logger.debug(
				`Conflict when moving '${oldPath}' to '${newPath}', the latter already exists, deconflicting by moving it to '${deconflictedPath}'`
			);

			const existingMetadata = this.database.getDocument(newPath);
			if (
				existingMetadata === undefined ||
				existingMetadata.documentId !== documentId
			) {
				await this.database.updatePath(newPath, deconflictedPath);
				await this.fs.rename(newPath, deconflictedPath);
			} else {
				await this.database.deleteDocument(newPath);
				await this.fs.delete(newPath);
			}
		} else {
			await this.createParentDirectories(newPath);
		}

		await this.fs.rename(oldPath, newPath);
	}

	public isFileEligibleForSync(path: RelativePath): boolean {
		return isFileTypeMergable(path);
	}

	private async createParentDirectories(path: string): Promise<void> {
		const components = path.split("/");
		if (components.length === 1) {
			return;
		}
		for (let i = 1; i < components.length; i++) {
			const parentDir = components.slice(0, i).join("/");
			if (!(await this.fs.exists(parentDir))) {
				await this.fs.createDirectory(parentDir);
			}
		}
	}

	private async deconflictPath(path: RelativePath): Promise<RelativePath> {
		const pathParts = path.split("/");
		const fileName = pathParts.pop();
		if (fileName == "" || fileName == null) {
			throw new Error(`Path '${path}' cannot be empty`);
		}

		let directory = pathParts.join("/");
		if (directory) {
			directory += "/";
		}

		const nameParts = fileName.split(".");
		const extension =
			nameParts.length > 1 ? "." + nameParts[nameParts.length - 1] : "";
		let stem = extension ? nameParts.slice(0, -1).join(".") : fileName;
		let currentCount = Number.parseInt(
			FileOperations.PARENTHESES_REGEX.exec(stem)?.groups?.[0] ?? "0"
		);
		stem = stem.replace(FileOperations.PARENTHESES_REGEX, "");

		// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
		while (true) {
			const newName =
				currentCount === 0
					? `${directory}${stem}${extension}`
					: `${directory}${stem} (${currentCount})${extension}`;
			if (await this.fs.exists(newName)) {
				currentCount++;
			} else {
				return newName;
			}
		}
	}
}
