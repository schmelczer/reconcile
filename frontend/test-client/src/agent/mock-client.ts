import type {
	RelativePath,
	FileSystemOperations,
	SyncSettings
} from "sync-client";
import { SyncClient } from "sync-client";
import { assert } from "../utils/assert";

export class MockClient implements FileSystemOperations {
	protected readonly localFiles = new Map<string, Uint8Array>();
	protected client!: SyncClient;
	protected data: object | undefined = undefined;

	public constructor(
		private readonly initialSettings: Partial<SyncSettings>
	) {}

	public async init(): Promise<void> {
		this.client = await SyncClient.create(this, {
			load: async () => this.data,
			save: async (data) => void (this.data = data)
		});

		await Promise.all(
			Object.keys(this.initialSettings).map(async (key) => {
				if (key in this.client.settings) {
					return this.client.settings.setSetting(
						key as keyof SyncSettings, // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
						this.initialSettings[key as keyof SyncSettings] // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
					);
				}
			})
		);

		assert(
			(await this.client.checkConnection()).isSuccessful,
			"Connection check failed"
		);
	}

	public async listAllFiles(): Promise<RelativePath[]> {
		return Array.from(this.localFiles.keys());
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		const file = this.localFiles.get(path);
		if (!file) {
			throw new Error(`File ${path} does not exist`);
		}
		return file;
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		return (await this.read(path)).length;
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		if (!this.localFiles.has(path)) {
			throw new Error(`File ${path} does not exist`);
		}
		return new Date();
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return this.localFiles.has(path);
	}

	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		if (this.localFiles.has(path)) {
			throw new Error(`File ${path} already exists`);
		}
		this.localFiles.set(path, newContent);
		void this.client.syncer.syncLocallyCreatedFile(path, new Date());
	}

	public async createDirectory(_path: RelativePath): Promise<void> {
		// This doesn't mean anything in our virtual FS representation
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: string) => string
	): Promise<string> {
		const file = this.localFiles.get(path);
		if (!file) {
			throw new Error(`File ${path} does not exist`);
		}
		const currentContent = new TextDecoder().decode(file);
		const newContent = updater(currentContent);
		const newContentUint8Array = new TextEncoder().encode(newContent);
		this.localFiles.set(path, newContentUint8Array);

		void this.client.syncer.syncLocallyUpdatedFile({
			relativePath: path,
			updateTime: new Date()
		});

		return newContent;
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		this.localFiles.set(path, content);

		void this.client.syncer.syncLocallyUpdatedFile({
			relativePath: path,
			updateTime: new Date()
		});
	}

	public async delete(path: RelativePath): Promise<void> {
		this.localFiles.delete(path);
		void this.client.syncer.syncLocallyDeletedFile(path);
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		const file = this.localFiles.get(oldPath);
		if (!file) {
			throw new Error(`File ${oldPath} does not exist`);
		}
		this.localFiles.set(newPath, file);
		if (oldPath !== newPath) {
			this.localFiles.delete(oldPath);
		}

		void this.client.syncer.syncLocallyUpdatedFile({
			oldPath,
			relativePath: newPath,
			updateTime: new Date()
		});
	}
}
