import type {
	RelativePath,
	FileSystemOperations,
	SyncSettings
} from "sync-client";
import { SyncClient } from "sync-client";
import { assert } from "../utils/assert";

export class MockClient implements FileSystemOperations {
	protected readonly localFiles: Record<string, Uint8Array> = {};
	protected client!: SyncClient;
	protected data: unknown = "";

	public constructor(
		private readonly initialSettings: Partial<SyncSettings>
	) {}

	public async init(): Promise<void> {
		this.client = await SyncClient.create(this, {
			load: async () => this.data,
			save: async (data: unknown) => void (this.data = data)
		});

		await Promise.all(
			Object.keys(this.initialSettings).map(async (key) => {
				return this.client.settings.setSetting(
					key as keyof SyncSettings,
					this.initialSettings[key as keyof SyncSettings]
				);
			})
		);

		assert(
			(await this.client.checkConnection()).isSuccessful,
			"Connection check failed"
		);
	}

	public async listAllFiles(): Promise<RelativePath[]> {
		return Object.keys(this.localFiles);
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		if (!(path in this.localFiles)) {
			throw new Error(`File ${path} does not exist`);
		}
		return this.localFiles[path];
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		if (!(path in this.localFiles)) {
			throw new Error(`File ${path} does not exist`);
		}
		return this.localFiles[path].length;
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		if (!(path in this.localFiles)) {
			throw new Error(`File ${path} does not exist`);
		}
		return new Date();
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return path in this.localFiles;
	}

	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		if (path in this.localFiles) {
			throw new Error(`File ${path} already exists`);
		}
		this.localFiles[path] = newContent;
		void this.client.syncer.syncLocallyCreatedFile(path, new Date());
	}

	public async createDirectory(path: RelativePath): Promise<void> {
		// This doesn't mean anything in our virtual FS representation
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: string) => string
	): Promise<string> {
		if (!(path in this.localFiles)) {
			throw new Error(`File ${path} does not exist`);
		}
		const currentContent = new TextDecoder().decode(this.localFiles[path]);
		const newContent = updater(currentContent);
		const newContentUint8Array = new TextEncoder().encode(newContent);
		this.localFiles[path] = newContentUint8Array;

		void this.client.syncer.syncLocallyUpdatedFile({
			relativePath: path,
			updateTime: new Date()
		});

		return newContent;
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		this.localFiles[path] = content;

		void this.client.syncer.syncLocallyUpdatedFile({
			relativePath: path,
			updateTime: new Date()
		});
	}

	public async delete(path: RelativePath): Promise<void> {
		delete this.localFiles[path];
		void this.client.syncer.syncLocallyDeletedFile(path);
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		if (!(oldPath in this.localFiles)) {
			throw new Error(`File ${oldPath} does not exist`);
		}

		this.localFiles[newPath] = this.localFiles[oldPath];
		if (oldPath !== newPath) {
			delete this.localFiles[oldPath];
		}

		void this.client.syncer.syncLocallyUpdatedFile({
			oldPath,
			relativePath: newPath,
			updateTime: new Date()
		});
	}
}
