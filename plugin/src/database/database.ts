import { Logger } from "src/logger";
import { DEFAULT_SETTINGS, SyncSettings } from "./sync-settings";
import {
	RelativePath,
	DocumentMetadata,
	VaultUpdateId,
} from "./document-metadata";

interface StoredDatabase {
	documents: Map<RelativePath, DocumentMetadata>;
	settings: SyncSettings;
	lastSeenUpdateId: VaultUpdateId | undefined;
}

export class Database {
	private _documents: Map<RelativePath, DocumentMetadata> = new Map();
	private _settings: SyncSettings;
	private _lastSeenUpdateId: VaultUpdateId | undefined;

	private onSettingsChangeHandlers: Array<(settings: SyncSettings) => void> =
		[];

	public constructor(
		initialState: Partial<StoredDatabase> | undefined,
		private saveData: (data: any) => Promise<void>
	) {
		initialState = initialState || {};
		if (
			Object.prototype.hasOwnProperty.call(initialState, "documents") &&
			initialState.documents
		) {
			for (const [relativePath, metadata] of Object.entries(
				initialState.documents
			)) {
				this._documents.set(relativePath, metadata as DocumentMetadata);
			}
		}

		Logger.getInstance().debug(
			`Loaded documents: ${JSON.stringify(
				Object.fromEntries(this._documents.entries()),
				null,
				2
			)}`
		);

		this._settings = Object.assign(
			{},
			DEFAULT_SETTINGS,
			initialState.settings || {}
		);

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
		this._settings = value;
		this.onSettingsChangeHandlers.forEach((handler) => handler(value));
		await this.save();
	}

	public addOnSettingsChangeHandlers(
		handler: (settings: SyncSettings) => void
	) {
		this.onSettingsChangeHandlers.push(handler);
	}

	public async setSetting<T extends keyof SyncSettings>(
		key: T,
		value: SyncSettings[T]
	): Promise<void> {
		this._settings[key] = value;
		Logger.getInstance().debug(
			`Setting ${key} to ${value}, new settings: ${JSON.stringify(
				this._settings
			)}`
		);
		await this.setSettings(this._settings);
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

	public async setDocument({
		relativePath,
		parentVersionId,
		hash,
	}: {
		relativePath: RelativePath;
		parentVersionId: VaultUpdateId;
		hash: string;
	}): Promise<void> {
		this._documents.set(relativePath, {
			parentVersionId,
			hash,
		});
		await this.save();
	}

	public async moveDocument({
		oldRelativePath,
		relativePath,
		parentVersionId,
		hash,
	}: {
		oldRelativePath: RelativePath;
		relativePath: RelativePath;
		parentVersionId: VaultUpdateId;
		hash: string;
	}): Promise<void> {
		this._documents.delete(oldRelativePath);
		this._documents.set(relativePath, {
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
