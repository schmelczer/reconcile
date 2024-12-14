export type VaultUpdateId = number;
export type RelativePath = string;

export interface DocumentMetadata {
	parentVersionId: VaultUpdateId;
	hash: string;
}
