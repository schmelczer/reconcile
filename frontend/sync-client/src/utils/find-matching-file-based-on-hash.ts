import type { DocumentMetadata, RelativePath } from "src/persistence/database";
import { EMPTY_HASH } from "./hash";

export function findMatchingFileBasedOnHash(
	contentHash: string,
	candidates: [RelativePath, DocumentMetadata][]
): [RelativePath, DocumentMetadata] | undefined {
	if (contentHash === EMPTY_HASH) {
		return undefined;
	}

	return candidates.find(([_, metadata]) => metadata.hash === contentHash);
}
