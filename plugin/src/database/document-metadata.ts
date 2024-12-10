export type DocumentId = string;
export type DocumentVersionId = number;
export type RelativePath = string;

export interface DocumentMetadata {
	documentId: DocumentId;
	parentVersionId: DocumentVersionId;
}
