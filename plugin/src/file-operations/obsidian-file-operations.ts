import { normalizePath, Vault } from "obsidian";
import { FileOperations } from "./file-operations";
import * as lib from "../../../backend/sync_lib/pkg/sync_lib.js";
import { isEqualBytes } from "src/utils/is-equal-bytes";
import { RelativePath } from "src/database/document-metadata";

export class ObsidianFileOperations implements FileOperations {
	public constructor(private vault: Vault) {}

	async listAllFiles(): Promise<RelativePath[]> {
		const files = this.vault.getFiles();
		return files.map((file) => file.path);
	}

	async read(path: RelativePath): Promise<Uint8Array> {
		return new Uint8Array(
			await this.vault.adapter.readBinary(normalizePath(path))
		);
	}

	async getModificationTime(path: RelativePath): Promise<Date> {
		return new Date(
			(await this.vault.adapter.stat(normalizePath(path)))!.mtime
		);
	}

	async write(
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
		} else {
			await this.vault.adapter.writeBinary(
				normalizePath(path),
				newContent
			);

			return newContent;
		}
	}

	async create(path: RelativePath, newContent: Uint8Array): Promise<void> {
		if (await this.vault.adapter.exists(normalizePath(path))) {
			await this.write(path, new Uint8Array(0), newContent);
			return;
		}

		await this.vault.adapter.writeBinary(normalizePath(path), newContent);
	}

	async remove(path: RelativePath): Promise<void> {
		if (await this.vault.adapter.exists(normalizePath(path))) {
			return this.vault.adapter.remove(normalizePath(path));
		}
	}

	async move(oldPath: RelativePath, newPath: RelativePath): Promise<void> {
		if (oldPath === newPath) {
			return;
		}

		this.vault.adapter.rename(
			normalizePath(oldPath),
			normalizePath(newPath)
		);
	}
}
