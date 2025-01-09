import type { Vault } from "obsidian";
import { TFile } from "obsidian";
import { normalizePath } from "obsidian";
import type { FileOperations } from "./file-operations";
import type { RelativePath } from "src/database/document-metadata";
import { isBinary, mergeText } from "sync_lib";

export class ObsidianFileOperations implements FileOperations {
	public constructor(private readonly vault: Vault) {}

	public async listAllFiles(): Promise<RelativePath[]> {
		const files = this.vault.getFiles();
		return files.map((file) => file.path);
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		const result = new Uint8Array(
			await this.vault.adapter.readBinary(normalizePath(path))
		);

		return result;
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		const file = await this.vault.adapter.stat(normalizePath(path));
		if (!file) {
			throw new Error(`File not found: ${path}`);
		}

		return file.size;
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		const file = await this.vault.adapter.stat(normalizePath(path));
		if (!file) {
			throw new Error(`File not found: ${path}`);
		}

		return new Date(file.mtime);
	}

	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		if (await this.vault.adapter.exists(normalizePath(path))) {
			await this.write(path, new Uint8Array(0), newContent);
			return;
		}

		await this.createParentDirectories(normalizePath(path));
		await this.vault.adapter.writeBinary(normalizePath(path), newContent);
	}

	public async write(
		path: RelativePath,
		expectedContent: Uint8Array,
		newContent: Uint8Array
	): Promise<Uint8Array> {
		if (!(await this.vault.adapter.exists(normalizePath(path)))) {
			// The caller assumed the file exists, but it doesn't, let's not recreate it
			return new Uint8Array(0);
		}

		if (isBinary(expectedContent)) {
			await this.vault.adapter.writeBinary(
				normalizePath(path),
				newContent
			);
			return newContent;
		}

		const expetedText = new TextDecoder().decode(expectedContent);
		const newText = new TextDecoder().decode(newContent);

		const resultText = await this.vault.adapter.process(
			normalizePath(path),
			(currentText) => {
				if (currentText !== expetedText) {
					return mergeText(expetedText, currentText, newText);
				}

				return newText;
			}
		);
		return new TextEncoder().encode(resultText);
	}

	public async remove(path: RelativePath): Promise<void> {
		if (await this.vault.adapter.exists(normalizePath(path))) {
			return this.vault.adapter.remove(normalizePath(path));
		}
	}

	public async move(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		if (oldPath === newPath) {
			return;
		}

		await this.vault.adapter.rename(
			normalizePath(oldPath),
			normalizePath(newPath)
		);
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
