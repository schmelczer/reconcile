export type VaultUpdateId = number;
export type DocumentId = string;
export type RelativePath = string;

export interface DocumentMetadata {
	parentVersionId: VaultUpdateId;
	hash: string;
}
