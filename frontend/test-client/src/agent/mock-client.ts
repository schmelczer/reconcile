import type { StoredDatabase, TextWithCursors } from "sync-client";
import { assert } from "../utils/assert";
import {
	type RelativePath,
	type FileSystemOperations,
	type SyncSettings,
	SyncClient
} from "sync-client";

export class MockClient implements FileSystemOperations {
	protected readonly localFiles = new Map<string, Uint8Array>();
	protected client!: SyncClient;

	protected data: Partial<{
		settings: Partial<SyncSettings>;
		database: Partial<StoredDatabase>;
	}> = {
		database: {
			// Assume all clients start at the same time so there's no need to fetch
			// any shared state.
			hasInitialSyncCompleted: true
		}
	};

	public constructor(
		initialSettings: Partial<SyncSettings>,
		protected readonly useSlowFileEvents: boolean
	) {
		this.data.settings = initialSettings;
	}

	public async init(
		fetchImplementation: typeof globalThis.fetch,
		webSocketImplementation: typeof globalThis.WebSocket
	): Promise<void> {
		this.client = await SyncClient.create({
			fs: this,
			persistence: {
				load: async () => this.data,
				save: async (data) => void (this.data = data)
			},
			fetch: fetchImplementation,
			webSocket: webSocketImplementation
		});

		await this.client.start();
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
		this.client.logger.info(
			`Creating file ${path} with content ${new TextDecoder().decode(newContent)}`
		);
		this.localFiles.set(path, newContent);

		this.executeFileOperation(() => {
			void this.client.syncLocallyCreatedFile(path);
		});
	}

	public async createDirectory(_path: RelativePath): Promise<void> {
		// This doesn't mean anything in our virtual FS representation
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (currentContent: TextWithCursors) => TextWithCursors
	): Promise<string> {
		const file = this.localFiles.get(path);
		if (!file) {
			throw new Error(`File ${path} does not exist`);
		}
		const currentContent = new TextDecoder().decode(file);
		const newContent = updater({ text: currentContent, cursors: [] }).text;
		const newContentUint8Array = new TextEncoder().encode(newContent);
		this.localFiles.set(path, newContentUint8Array);

		if (!this.useSlowFileEvents) {
			const existingParts = currentContent
				.split(" ")
				.map((part) => part.trim());
			const newParts = newContent.split(" ").map((part) => part.trim());
			existingParts.forEach((part) =>
				// all changes should be additive
				{
					assert(
						newParts.includes(part),
						`Part ${part} not found in new content: ${newContent}`
					);
				}
			);
		}

		this.client.logger.info(
			`Updated file ${path} with:\n  current content: ${currentContent}\n  new content: ${newContent}`
		);

		this.executeFileOperation(() => {
			void this.client.syncLocallyUpdatedFile({
				relativePath: path
			});
		});

		return newContent;
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		const hasExisted = this.localFiles.has(path);
		this.localFiles.set(path, content);

		this.client.logger.info(
			`Updated file ${path} with:\n  new content: ${new TextDecoder().decode(content)}`
		);

		this.executeFileOperation(() => {
			if (hasExisted) {
				void this.client.syncLocallyUpdatedFile({
					relativePath: path
				});
			} else {
				void this.client.syncLocallyCreatedFile(path);
			}
		});
	}

	public async delete(path: RelativePath): Promise<void> {
		this.client.logger.info(
			`Deleting file: ${path} with:\n  content ${new TextDecoder().decode(this.localFiles.get(path))}`
		);
		this.localFiles.delete(path);

		this.executeFileOperation(() => {
			void this.client.syncLocallyDeletedFile(path);
		});
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

		this.client.logger.info(
			`Renamed file: ${oldPath} -> ${newPath} with:\n  content ${new TextDecoder().decode(file)}`
		);

		this.executeFileOperation(() => {
			void this.client.syncLocallyUpdatedFile({
				oldPath,
				relativePath: newPath
			});
		});
	}

	private executeFileOperation(callback: () => void): void {
		if (this.useSlowFileEvents) {
			// we aren't the best client and it takes some time to notice changes
			setTimeout(callback, Math.random() * 100);
		} else {
			callback();
		}
	}
}
