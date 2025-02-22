export type VaultUpdateId = number;
export type DocumentId = string;
export type RelativePath = string;

export interface DocumentMetadata {
	parentVersionId: VaultUpdateId;
	documentId: DocumentId;
	hash: string;
}

import { Logger } from "src/tracing/logger";

export interface StoredDatabase {
	documents: Map<RelativePath, DocumentMetadata>;
	lastSeenUpdateId: VaultUpdateId | undefined;
}

export class Database {
	private documents = new Map<RelativePath, DocumentMetadata>();
	private lastSeenUpdateId: VaultUpdateId | undefined;

	public constructor(
		private readonly logger: Logger,
		initialState: Partial<StoredDatabase> | undefined,
		private readonly saveData: (data: unknown) => Promise<void>
	) {
		initialState ??= {};
		if (initialState.documents) {
			for (const [relativePath, metadata] of Object.entries(
				initialState.documents
			)) {
				// eslint-disable-next-line @typescript-eslint/no-unsafe-type-assertion
				this.documents.set(relativePath, metadata as DocumentMetadata);
			}
		}
		this.logger.debug(`Loaded ${this.documents.size} documents`);

		this.lastSeenUpdateId = initialState.lastSeenUpdateId;
		this.logger.debug(
			`Loaded last seen update id: ${this.lastSeenUpdateId}`
		);
	}

	public getDocuments(): Map<RelativePath, DocumentMetadata> {
		return this.documents;
	}

	public getLastSeenUpdateId(): VaultUpdateId | undefined {
		return this.lastSeenUpdateId;
	}

	public async setLastSeenUpdateId(
		value: VaultUpdateId | undefined
	): Promise<void> {
		this.lastSeenUpdateId = value;
		await this.save();
	}

	public async resetSyncState(): Promise<void> {
		this.documents = new Map();
		this.lastSeenUpdateId = 0;
		await this.save();
	}

	public getDocumentByDocumentId(
		documentId: DocumentId
	): [RelativePath, DocumentMetadata] | undefined {
		return [...this.documents.entries()].find(
			([_, metadata]) => metadata.documentId === documentId
		);
	}

	public async setDocument({
		documentId,
		relativePath,
		parentVersionId,
		hash
	}: {
		documentId: DocumentId;
		relativePath: RelativePath;
		parentVersionId: VaultUpdateId;
		hash: string;
	}): Promise<void> {
		this.documents.set(relativePath, {
			documentId,
			parentVersionId,
			hash
		});
		await this.save();
	}

	public async moveDocument({
		documentId,
		oldRelativePath,
		relativePath,
		parentVersionId,
		hash
	}: {
		documentId: DocumentId;
		oldRelativePath: RelativePath;
		relativePath: RelativePath;
		parentVersionId: VaultUpdateId;
		hash: string;
	}): Promise<void> {
		this.documents.delete(oldRelativePath);
		this.documents.set(relativePath, {
			documentId,
			parentVersionId,
			hash
		});
		await this.save();
	}

	public async removeDocument(relativePath: RelativePath): Promise<void> {
		this.documents.delete(relativePath);
		await this.save();
	}

	public getDocument(
		relativePath: RelativePath
	): DocumentMetadata | undefined {
		return this.documents.get(relativePath);
	}

	private async save(): Promise<void> {
		await this.saveData({
			documents: Object.fromEntries(this.documents.entries()),
			lastSeenUpdateId: this.lastSeenUpdateId
		});
	}
}
