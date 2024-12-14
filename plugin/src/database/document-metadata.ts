export type DocumentVersionId = number;
export type RelativePath = string;

export interface DocumentMetadata {
	parentVersionId: DocumentVersionId;
	hash: string;
}
