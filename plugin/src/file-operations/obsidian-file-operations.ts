import type { Vault } from "obsidian";
import { normalizePath } from "obsidian";
import type { FileOperations } from "./file-operations";
import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { isEqualBytes } from "src/utils/is-equal-bytes";
import type { RelativePath } from "src/database/document-metadata";

export class ObsidianFileOperations implements FileOperations {
	public constructor(private readonly vault: Vault) {}

	public async listAllFiles(): Promise<RelativePath[]> {
		const files = this.vault.getFiles();
		return files.map((file) => file.path);
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		return new Uint8Array(
			await this.vault.adapter.readBinary(normalizePath(path))
		);
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		const file = await this.vault.adapter.stat(normalizePath(path));
		if (!file) {
			throw new Error(`File not found: ${path}`);
		}
		return new Date(file.mtime);
	}

	public async write(
		path: RelativePath,
		expectedContent: Uint8Array,
		newContent: Uint8Array
	): Promise<Uint8Array> {
		if (!(await this.vault.adapter.exists(normalizePath(path)))) {
			return new Uint8Array(0);
		}

		const currentContent = await this.read(path);
		if (!isEqualBytes(currentContent, expectedContent)) {
			const result = lib.merge(
				expectedContent,
				currentContent,
				newContent
			);

			await this.vault.adapter.writeBinary(normalizePath(path), result);

			return result;
		}
		await this.vault.adapter.writeBinary(normalizePath(path), newContent);

		return newContent;
	}

	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		if (await this.vault.adapter.exists(normalizePath(path))) {
			await this.write(path, new Uint8Array(0), newContent);
			return;
		}

		await this.vault.adapter.writeBinary(normalizePath(path), newContent);
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
}
