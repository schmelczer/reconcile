import { normalizePath, Stat, Vault } from "obsidian";
import { FileSystemOperations, RelativePath } from "sync-client";

export class ObsidianFileSystemOperations implements FileSystemOperations {
	public constructor(private readonly vault: Vault) {}

	public async listAllFiles(): Promise<RelativePath[]> {
		return this.vault.getFiles().map((file) => file.path);
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		return new Uint8Array(
			await this.vault.adapter.readBinary(normalizePath(path))
		);
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		return this.vault.adapter.writeBinary(
			normalizePath(path),
			content.buffer as ArrayBuffer
		);
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: string) => string
	): Promise<string> {
		return this.vault.adapter.process(normalizePath(path), updater);
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		return (await this.statFile(path)).size;
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		return new Date((await this.statFile(path)).mtime);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return this.vault.adapter.exists(normalizePath(path));
	}

	public async createDirectory(path: RelativePath): Promise<void> {
		return this.vault.adapter.mkdir(normalizePath(path));
	}

	public async delete(path: RelativePath): Promise<void> {
		if (!(await this.vault.adapter.trashSystem(normalizePath(path)))) {
			return this.vault.adapter.remove(normalizePath(path));
		}
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		return this.vault.adapter.rename(oldPath, newPath);
	}

	private async statFile(path: string): Promise<Stat> {
		const file = await this.vault.adapter.stat(normalizePath(path));

		if (!file) {
			throw new Error(`File not found: ${path}`);
		}

		return file;
	}
}
