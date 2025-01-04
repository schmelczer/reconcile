import type { SyncSettings } from "./sync-settings";
import { DEFAULT_SETTINGS } from "./sync-settings";
import type {
	DocumentId,
	DocumentMetadata,
	RelativePath,
	VaultUpdateId,
} from "./document-metadata";
import { Logger } from "src/tracing/logger";

interface StoredDatabase {
	documents: Map<RelativePath, DocumentMetadata>;
	settings: SyncSettings;
	lastSeenUpdateId: VaultUpdateId | undefined;
}

// Todo: split it into settings and documents
export class Database {
	private _documents = new Map<RelativePath, DocumentMetadata>();
	private _settings: SyncSettings;
	private _lastSeenUpdateId: VaultUpdateId | undefined;

	private readonly onSettingsChangeHandlers: ((
		newSettings: SyncSettings,
		oldSettings: SyncSettings
	) => void)[] = [];

	public constructor(
		initialState: Partial<StoredDatabase> | undefined,
		private readonly saveData: (data: unknown) => Promise<void>
	) {
		initialState ??= {};
		if (
			// eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
			Object.prototype.hasOwnProperty.call(initialState, "documents") &&
			initialState.documents
		) {
			for (const [relativePath, metadata] of Object.entries(
				initialState.documents
			)) {
				// eslint-disable-next-line @typescript-eslint/no-unsafe-type-assertion
				this._documents.set(relativePath, metadata as DocumentMetadata);
			}
		}

		Logger.getInstance().debug(`Loaded ${this._documents.size} documents`);

		this._settings = {
			...DEFAULT_SETTINGS,
			...(initialState.settings ?? {}),
		};

		Logger.getInstance().debug(
			`Loaded settings: ${JSON.stringify(this._settings, null, 2)}`
		);

		this._lastSeenUpdateId = initialState.lastSeenUpdateId;

		Logger.getInstance().debug(
			`Loaded last seen update id: ${this._lastSeenUpdateId}`
		);
	}

	public getDocuments(): Map<RelativePath, DocumentMetadata> {
		return this._documents;
	}

	public getSettings(): SyncSettings {
		return this._settings;
	}

	public async setSettings(value: SyncSettings): Promise<void> {
		const oldSettings = this._settings;
		this._settings = value;
		this.onSettingsChangeHandlers.forEach((handler) => {
			handler(value, oldSettings);
		});
		await this.save();
	}

	public addOnSettingsChangeHandlers(
		handler: (settings: SyncSettings, oldSettings: SyncSettings) => void
	): void {
		this.onSettingsChangeHandlers.push(handler);
	}

	public async setSetting<T extends keyof SyncSettings>(
		key: T,
		value: SyncSettings[T]
	): Promise<void> {
		let newSettings = { ...this._settings, [key]: value };
		Logger.getInstance().debug(
			`Setting ${key} to ${value}, new settings: ${JSON.stringify(
				newSettings,
				null,
				2
			)}`
		);
		await this.setSettings(newSettings);
	}

	public getLastSeenUpdateId(): VaultUpdateId | undefined {
		return this._lastSeenUpdateId;
	}

	public async setLastSeenUpdateId(
		value: VaultUpdateId | undefined
	): Promise<void> {
		this._lastSeenUpdateId = value;
		await this.save();
	}

	public async resetSyncState(): Promise<void> {
		this._documents = new Map();
		this._lastSeenUpdateId = 0;
		await this.save();
	}

	public getDocumentByDocumentId(
		documentId: DocumentId
	): [RelativePath, DocumentMetadata] | undefined {
		return [...this._documents.entries()].find(
			([_, metadata]) => metadata.documentId === documentId
		);
	}

	public async setDocument({
		documentId,
		relativePath,
		parentVersionId,
		hash,
	}: {
		documentId: DocumentId;
		relativePath: RelativePath;
		parentVersionId: VaultUpdateId;
		hash: string;
	}): Promise<void> {
		this._documents.set(relativePath, {
			documentId,
			parentVersionId,
			hash,
		});
		await this.save();
	}

	public async moveDocument({
		documentId,
		oldRelativePath,
		relativePath,
		parentVersionId,
		hash,
	}: {
		documentId: DocumentId;
		oldRelativePath: RelativePath;
		relativePath: RelativePath;
		parentVersionId: VaultUpdateId;
		hash: string;
	}): Promise<void> {
		this._documents.delete(oldRelativePath);
		this._documents.set(relativePath, {
			documentId,
			parentVersionId,
			hash,
		});
		await this.save();
	}

	public async removeDocument(relativePath: RelativePath): Promise<void> {
		this._documents.delete(relativePath);
		await this.save();
	}

	public getDocument(
		relativePath: RelativePath
	): DocumentMetadata | undefined {
		return this._documents.get(relativePath);
	}

	private async save(): Promise<void> {
		await this.saveData({
			documents: Object.fromEntries(this._documents.entries()),
			settings: this._settings,
			lastSeenUpdateId: this._lastSeenUpdateId,
		});
	}
}
