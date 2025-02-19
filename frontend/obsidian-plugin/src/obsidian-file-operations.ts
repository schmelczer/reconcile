import type { Stat, Vault } from "obsidian";
import { normalizePath } from "obsidian";
import { Platform } from "obsidian";
import type { FileOperations, RelativePath } from "sync-client";
import { Logger, isFileTypeMergable, mergeText } from "sync-client";

export class ObsidianFileOperations implements FileOperations {
	public constructor(private readonly vault: Vault) {}

	public async listAllFiles(): Promise<RelativePath[]> {
		const files = this.vault.getFiles();
		Logger.getInstance().debug(`Listing all files, found ${files.length}`);
		return files.map((file) => file.path);
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		Logger.getInstance().debug(`Reading file: ${path}`);
		if (isFileTypeMergable(path)) {
			let text = await this.vault.adapter.read(normalizePath(path));

			text = text.replace(/\r\n/g, "\n");

			return new TextEncoder().encode(text);
		}
		return new Uint8Array(
			await this.vault.adapter.readBinary(normalizePath(path))
		);
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		Logger.getInstance().debug(`Getting file size: ${path}`);
		return (await this.statFile(path)).size;
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		Logger.getInstance().debug(`Getting modification time: ${path}`);
		return new Date((await this.statFile(path)).mtime);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		Logger.getInstance().debug(`Checking existance of ${path}`);
		return this.vault.adapter.exists(normalizePath(path));
	}

	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		Logger.getInstance().debug(`Creating file: ${path}`);
		if (await this.vault.adapter.exists(normalizePath(path))) {
			Logger.getInstance().debug(
				`Didn't expect ${path} to exist, when trying to create it, merging instead`
			);
			await this.write(path, new Uint8Array(0), newContent);
			return;
		}

		await this.createParentDirectories(normalizePath(path));
		await this.vault.adapter.writeBinary(
			normalizePath(path),
			newContent.buffer as ArrayBuffer
		);
	}

	public async write(
		path: RelativePath,
		expectedContent: Uint8Array,
		newContent: Uint8Array
	): Promise<Uint8Array> {
		Logger.getInstance().debug(`Writing file: ${path}`);
		if (!(await this.vault.adapter.exists(normalizePath(path)))) {
			Logger.getInstance().debug(
				`The caller assumed ${path} exists, but it no longer, so we wont recreate it`
			);
			return new Uint8Array(0);
		}

		if (!isFileTypeMergable(path)) {
			Logger.getInstance().debug(
				`The expected content is not mergable, so we won't perform a 3-way merge, just overwrite it`
			);
			await this.vault.adapter.writeBinary(
				normalizePath(path),
				newContent.buffer as ArrayBuffer
			);
			return newContent;
		}

		const expetedText = new TextDecoder().decode(expectedContent);
		const newText = new TextDecoder().decode(newContent);

		const resultText = await this.vault.adapter.process(
			normalizePath(path),
			(currentText) => {
				currentText = currentText.replace(/\r\n/g, "\n");
				if (currentText !== expetedText) {
					Logger.getInstance().debug(
						`Performing a 3-way merge for ${path} with the expected content`
					);

					return mergeText(expetedText, currentText, newText);
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
		if (await this.vault.adapter.exists(normalizePath(path))) {
			await this.vault.adapter.trashSystem(normalizePath(path));
		}
	}

	public async move(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		oldPath = normalizePath(oldPath);
		newPath = normalizePath(newPath);

		Logger.getInstance().debug(`Moving file: ${oldPath} -> ${newPath}`);

		if (oldPath === newPath) {
			return;
		}

		await this.createParentDirectories(newPath);
		await this.vault.adapter.rename(oldPath, newPath);
	}

	public isFileEligibleForSync(path: RelativePath): boolean {
		if (Platform.isDesktopApp) {
			return true;
		}

		return isFileTypeMergable(path);
	}

	private async statFile(path: string): Promise<Stat> {
		const file = await this.vault.adapter.stat(normalizePath(path));

		if (!file) {
			throw new Error(`File not found: ${path}`);
		}

		return file;
	}

	private async createParentDirectories(path: string): Promise<void> {
		const components = path.split("/");
		if (components.length === 1) {
			return;
		}
		for (let i = 1; i < components.length; i++) {
			const parentDir = components.slice(0, i).join("/");
			if (!(await this.vault.adapter.exists(parentDir))) {
				await this.vault.adapter.mkdir(parentDir);
			}
		}
	}
}
