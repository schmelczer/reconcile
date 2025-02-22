import {
	SyncClient,
	RelativePath,
	FileSystemOperations,
	SyncSettings
} from "sync-client";
import { assert } from "../utils/assert";

export class MockClient implements FileSystemOperations {
	protected readonly files: Record<string, Uint8Array> = {};
	protected client!: SyncClient;

	public constructor(
		protected readonly globalFiles: Record<string, Uint8Array>,
		private readonly initialSettings: Partial<SyncSettings>
	) {}

	public async init() {
		let _data: unknown = "";

		this.client = await SyncClient.create(this, {
			load: async () => _data,
			save: async (data: unknown) => void (_data = data)
		});

		Object.keys(this.initialSettings).forEach((key) => {
			this.client.settings.setSetting(
				key as keyof SyncSettings,
				this.initialSettings[key as keyof SyncSettings]
			);
		});

		assert(
			(await this.client.checkConnection()).isSuccessful,
			"Connection check failed"
		);
	}

	public async listAllFiles(): Promise<RelativePath[]> {
		return Object.keys(this.files) as RelativePath[];
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		return this.files[path];
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		return this.files[path].length;
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		return new Date();
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return path in this.files;
	}

	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		this.globalFiles[path] = newContent;
		this.files[path] = newContent;
		this.client.syncer.syncLocallyCreatedFile(path, new Date());
	}

	public async createDirectory(path: RelativePath): Promise<void> {}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: string) => string
	): Promise<string> {
		const currentContent = new TextDecoder().decode(this.files[path]);
		const newContent = updater(currentContent);
		const newContentUint8Array = new TextEncoder().encode(newContent);
		this.globalFiles[path] = newContentUint8Array;
		this.files[path] = newContentUint8Array;
		this.client.syncer.syncLocallyUpdatedFile({
			relativePath: path,
			updateTime: new Date()
		});
		return newContent;
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		this.globalFiles[path] = content;
		this.files[path] = content;
		this.client.syncer.syncLocallyUpdatedFile({
			relativePath: path,
			updateTime: new Date()
		});
	}

	public async delete(path: RelativePath): Promise<void> {
		delete this.files[path];
		if (path in this.globalFiles) {
			delete this.globalFiles[path];
		}
		this.client.syncer.syncLocallyDeletedFile(path);
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		this.files[newPath] = this.files[oldPath];
		delete this.files[oldPath];

		if (oldPath in this.globalFiles) {
			this.globalFiles[newPath] = this.files[oldPath];
			delete this.globalFiles[oldPath];
		}

		this.client.syncer.syncLocallyUpdatedFile({
			oldPath,
			relativePath: newPath,
			updateTime: new Date()
		});
	}

	public isFileEligibleForSync(path: RelativePath): boolean {
		return true;
	}
}
