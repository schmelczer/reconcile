import { Logger } from "src/tracing/logger";
import { FileSystemOperations } from "./filesystem-operations";
import { RelativePath } from "src/persistence/database";
import { isBinary, isFileTypeMergable, mergeText } from "sync_lib";

export class FileOperations {
	public constructor(private readonly fs: FileSystemOperations) {}

	public async listAllFiles(): Promise<RelativePath[]> {
		const files = await this.fs.listAllFiles();
		Logger.getInstance().debug(`Listing all files, found ${files.length}`);
		return files;
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		Logger.getInstance().debug(`Reading file: ${path}`);
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
		Logger.getInstance().debug(`Getting file size: ${path}`);
		return this.fs.getFileSize(path);
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		Logger.getInstance().debug(`Getting modification time: ${path}`);
		return this.fs.getModificationTime(path);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		Logger.getInstance().debug(`Checking existance of ${path}`);
		return this.fs.exists(path);
	}

	// Create and write the file if it doesn't exist. Otherwise, it has the same behavior as write.
	// All parent directories are created if they don't exist.
	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		Logger.getInstance().debug(`Creating file: ${path}`);
		if (await this.fs.exists(path)) {
			Logger.getInstance().debug(
				`Didn't expect ${path} to exist, when trying to create it, merging instead`
			);
			await this.write(path, new Uint8Array(0), newContent);
			return;
		}

		await this.createParentDirectories(path);
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
		Logger.getInstance().debug(`Writing file: ${path}`);
		if (!(await this.fs.exists(path))) {
			Logger.getInstance().debug(
				`The caller assumed ${path} exists, but it no longer, so we wont recreate it`
			);
			return new Uint8Array(0);
		}

		if (
			!isFileTypeMergable(path) ||
			isBinary(expectedContent) ||
			isBinary(newContent)
		) {
			Logger.getInstance().debug(
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
					Logger.getInstance().debug(
						`Performing a 3-way merge for ${path} with the expected content`
					);

					return mergeText(expectedText, currentText, newText);
				}

				Logger.getInstance().debug(
					`The current content of ${path} is the same as the expected content, so we will just write the new content`
				);

				return newText;
			}
		);
		return new TextEncoder().encode(resultText);
	}

	public async remove(path: RelativePath): Promise<void> {
		Logger.getInstance().debug(`Removing file: ${path}`);
		return this.fs.delete(path);
	}

	public async move(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		Logger.getInstance().debug(`Moving file: ${oldPath} -> ${newPath}`);

		if (oldPath === newPath) {
			return;
		}

		await this.createParentDirectories(newPath);
		await this.fs.rename(oldPath, newPath);
	}

	public isFileEligibleForSync(path: RelativePath): boolean {
		return true;
		// TODO: figure this out
		// if (Platform.isDesktopApp) {
		// 	return true;
		// }

		// return isFileTypeMergable(path);
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
}
