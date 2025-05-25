import type { Logger } from "../tracing/logger";
import { EMPTY_HASH } from "../utils/hash";
import { CoveredValues } from "../utils/min-covered";

export type VaultUpdateId = number;
export type DocumentId = string;
export type RelativePath = string;

export interface DocumentMetadata {
	parentVersionId: VaultUpdateId;
	hash: string;
	remoteRelativePath?: RelativePath;
}

export interface StoredDocumentMetadata {
	relativePath: RelativePath;
	documentId: DocumentId;
	parentVersionId: VaultUpdateId;
	remoteRelativePath?: RelativePath;
	hash: string;
}

export interface StoredDatabase {
	documents: StoredDocumentMetadata[];
	lastSeenUpdateId: VaultUpdateId | undefined;
	hasInitialSyncCompleted: boolean;
}

/**
 * Represents a document in the database.
 *
 * It is mutable and its content should always represent the latest
 * state of the document on disk based on the update events we have seen.
 */
export interface DocumentRecord {
	relativePath: RelativePath;
	documentId: DocumentId;
	metadata: DocumentMetadata | undefined;
	isDeleted: boolean;
	updates: Promise<void>[];
	parallelVersion: number;
}

export class Database {
	private documents: DocumentRecord[];
	private lastSeenUpdateIds: CoveredValues;
	private hasInitialSyncCompleted: boolean;

	public constructor(
		private readonly logger: Logger,
		initialState: Partial<StoredDatabase> | undefined,
		private readonly saveData: (data: StoredDatabase) => Promise<void>
	) {
		initialState ??= {};

		this.documents =
			initialState.documents?.map(
				({ relativePath, documentId, ...metadata }) => ({
					relativePath,
					documentId,
					metadata,
					isDeleted: false,
					updates: [],
					parallelVersion: 0
				})
			) ?? [];

		this.ensureConsistency();
		this.logger.debug(`Loaded ${this.documents.length} documents`);

		const { lastSeenUpdateId } = initialState;
		this.logger.debug(`Loaded last seen update id: ${lastSeenUpdateId}`);
		this.lastSeenUpdateIds = new CoveredValues(
			Math.max(0, lastSeenUpdateId ?? 0) // the first updateId will be 1 which is the first integer after -1
		);

		this.hasInitialSyncCompleted =
			initialState.hasInitialSyncCompleted ?? false;
		this.logger.debug(
			`Loaded hasInitialSyncCompleted: ${this.hasInitialSyncCompleted}`
		);
	}

	public get length(): number {
		return this.documents.length;
	}

	public get resolvedDocuments(): DocumentRecord[] {
		const paths = new Map<string, DocumentRecord[]>();
		this.documents
			.filter(({ metadata }) => metadata !== undefined)
			.forEach((record) =>
				paths.set(record.relativePath, [
					record,
					...(paths.get(record.relativePath) ?? [])
				])
			);

		return Array.from(paths.values()).map((records) => {
			records.sort(
				(a, b) => b.parallelVersion - a.parallelVersion // descending
			);

			if (
				records.length > 1 &&
				records.some((current, i) =>
					i === 0
						? false
						: records[i - 1].parallelVersion ===
							current.parallelVersion
				)
			) {
				throw new Error(
					`Multiple documents with the same parallel version and path at ${records[0].relativePath}`
				);
			}
			return records[0];
		});
	}

	public updateDocumentMetadata(
		metadata: {
			parentVersionId: VaultUpdateId;
			hash: string;
			remoteRelativePath: RelativePath;
		},
		toUpdate: DocumentRecord
	): void {
		if (!this.documents.includes(toUpdate)) {
			throw new Error("Document not found in database");
		}

		toUpdate.metadata = metadata;

		this.save();
	}

	public removeDocumentPromise(promise: Promise<void>): void {
		const entry = this.documents.find(({ updates }) =>
			updates.includes(promise)
		);

		if (entry === undefined) {
			// This method should be idempotent and tolerant of
			// stragglers calling it after the databse has been reset.
			return;
		}

		entry.updates = entry.updates.filter((update) => update !== promise);
		// No need to save as Promises don't get serialized
	}

	public removeDocument(find: DocumentRecord): void {
		this.documents = this.documents.filter((document) => document !== find);
		this.save();
	}

	public getLatestDocumentByRelativePath(
		find: RelativePath
	): DocumentRecord | undefined {
		const candidates = this.documents.filter(
			({ relativePath }) => relativePath === find
		);
		candidates.sort((a, b) => b.parallelVersion - a.parallelVersion); // descending
		return candidates[0];
	}

	public async getResolvedDocumentByRelativePath(
		relativePath: RelativePath,
		promise: Promise<void>
	): Promise<DocumentRecord> {
		const entry = this.getLatestDocumentByRelativePath(relativePath);

		if (entry === undefined) {
			throw new Error(
				`Document not found by relative path: ${relativePath}, ${JSON.stringify(
					this.documents,
					null,
					2
				)}`
			);
		}

		const currentPromises = entry.updates;
		entry.updates = [...currentPromises, promise];
		await Promise.all(currentPromises);

		return entry;
	}

	public createNewPendingDocument(
		documentId: DocumentId,
		relativePath: RelativePath,
		promise: Promise<void>
	): DocumentRecord {
		const previousEntry =
			this.getLatestDocumentByRelativePath(relativePath);

		const entry = {
			relativePath,
			documentId,
			metadata: undefined,
			isDeleted: false,
			updates: [promise],
			parallelVersion:
				previousEntry?.parallelVersion === undefined
					? 0
					: previousEntry.parallelVersion + 1
		};

		this.documents.push(entry);
		this.save();

		return entry;
	}

	public createNewEmptyDocument(
		documentId: DocumentId,
		parentVersionId: VaultUpdateId,
		relativePath: RelativePath
	): DocumentRecord {
		const entry = {
			relativePath,
			documentId,
			metadata: {
				parentVersionId,
				hash: EMPTY_HASH,
				remoteRelativePath: relativePath
			},
			isDeleted: false,
			updates: [],
			parallelVersion: 0
		};

		this.documents.push(entry);
		this.save();

		return entry;
	}

	public getDocumentByDocumentId(
		find: DocumentId
	): DocumentRecord | undefined {
		return this.documents.find(({ documentId }) => documentId === find);
	}

	public move(
		oldRelativePath: RelativePath,
		newRelativePath: RelativePath
	): void {
		const oldDocument =
			this.getLatestDocumentByRelativePath(oldRelativePath);

		if (oldDocument === undefined) {
			return;
		}

		const newDocument =
			this.getLatestDocumentByRelativePath(newRelativePath);
		if (newDocument?.isDeleted === false) {
			throw new Error(
				`Document already exists at new location: ${newRelativePath}`
			);
		}

		oldDocument.relativePath = newRelativePath;
		// We're in a strange state where the target of the move has just got deleted,
		// however, its metadata might already have a bunch of updates queued up for
		// the document at the new location. We need to keep these updates.
		oldDocument.parallelVersion =
			newDocument !== undefined ? newDocument.parallelVersion + 1 : 0;

		this.save();
	}

	public delete(relativePath: RelativePath): void {
		const candidate = this.getLatestDocumentByRelativePath(relativePath);
		if (candidate === undefined) {
			throw new Error(
				`Document not found by relative path: ${relativePath}`
			);
		}
		candidate.isDeleted = true;
	}

	public getHasInitialSyncCompleted(): boolean {
		return this.hasInitialSyncCompleted;
	}

	public setHasInitialSyncCompleted(value: boolean): void {
		this.hasInitialSyncCompleted = value;
		this.save();
	}

	public getLastSeenUpdateId(): VaultUpdateId {
		return this.lastSeenUpdateIds.min;
	}

	public addSeenUpdateId(value: number): void {
		const previousMin = this.lastSeenUpdateIds.min;
		this.lastSeenUpdateIds.add(value);
		if (previousMin !== this.lastSeenUpdateIds.min) {
			this.save();
		}
	}

	public setLastSeenUpdateId(value: number): void {
		this.lastSeenUpdateIds.min = value;
		this.save();
	}

	public reset(): void {
		this.documents = [];
		this.lastSeenUpdateIds = new CoveredValues(
			0 // the first updateId will be 1 which is the first integer after -1
		);
		this.hasInitialSyncCompleted = false;
		this.save();
	}

	private save(): void {
		this.ensureConsistency();
		void this.saveData({
			documents: this.resolvedDocuments.map(
				({ relativePath, documentId, metadata }) => ({
					documentId,
					relativePath,
					// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
					...metadata! // `resolvedDocuments` only returns docs with metadata set
				})
			),
			lastSeenUpdateId: this.lastSeenUpdateIds.min,
			hasInitialSyncCompleted: this.hasInitialSyncCompleted
		});
	}

	private ensureConsistency(): void {
		const idToPath = new Map<string, string[]>();

		this.resolvedDocuments.forEach(({ relativePath, documentId }) => {
			idToPath.set(documentId, [
				...(idToPath.get(documentId) ?? []),
				relativePath
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
