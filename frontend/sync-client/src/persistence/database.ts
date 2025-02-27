export type VaultUpdateId = number;
export type DocumentId = string;
export type RelativePath = string;

export interface DocumentMetadata {
	parentVersionId: VaultUpdateId;
	documentId: DocumentId;
	hash: string;
}

import type { Logger } from "src/tracing/logger";

export interface StoredDatabase {
	documents: Record<RelativePath, DocumentMetadata>;
	lastSeenUpdateId: VaultUpdateId | undefined;
}

export class Database {
	private documents = new Map<RelativePath, DocumentMetadata>();

	private lastSeenUpdateId: VaultUpdateId | undefined;

	public constructor(
		private readonly logger: Logger,
		initialState: Partial<StoredDatabase> | undefined,
		private readonly saveData: (data: StoredDatabase) => Promise<void>
	) {
		initialState ??= {};
		if (initialState.documents) {
			for (const [relativePath, metadata] of Object.entries(
				initialState.documents
			)) {
				this.documents.set(relativePath, metadata);
			}
		}
		this.ensureConsistency();

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

	public async removeDocument(relativePath: RelativePath): Promise<void> {
		this.documents.delete(relativePath);
		await this.save();
	}

	public getDocument(
		relativePath: RelativePath
	): DocumentMetadata | undefined {
		return this.documents.get(relativePath);
	}

	public async deleteDocument(relativePath: RelativePath): Promise<void> {
		this.documents.delete(relativePath);
		await this.save();
	}

	public async updatePath(
		oldRelativePath: RelativePath,
		newRelativePath: RelativePath
	): Promise<void> {
		const document = this.documents.get(oldRelativePath);
		if (!document) {
			throw new Error(
				`Cannot update physical path for document that does not exist: ${oldRelativePath}`
			);
		}

		if (this.documents.has(newRelativePath)) {
			throw new Error(
				`Cannot update physical path to path that is already in use: ${newRelativePath}`
			);
		}

		this.documents.delete(oldRelativePath);
		this.documents.set(newRelativePath, document);

		await this.save();
	}

	private async save(): Promise<void> {
		this.ensureConsistency();
		await this.saveData({
			documents: Object.fromEntries(this.documents.entries()),
			lastSeenUpdateId: this.lastSeenUpdateId
		});
	}

	private ensureConsistency(): void {
		const allMetadata = Array.from(this.documents.entries());
		const idToPath = new Map<string, Array<string>>();

		allMetadata.forEach(([name, metadata]) => {
			idToPath.set(metadata.documentId, [
				...(idToPath.get(metadata.documentId) ?? []),
				name
			]);
		});

		const duplicates = Array.from(idToPath.entries())
			.filter(([_, paths]) => paths.length > 1)
			.map(([id, paths]) => `${id} (${paths.join(", ")})`);

		if (duplicates.length > 0) {
			throw new Error(
				"Document IDs are not unique, found duplicates: " +
					duplicates.join("; ")
			);
		}
	}
}
