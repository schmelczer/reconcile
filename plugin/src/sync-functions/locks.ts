import { DocumentId } from "src/database/document-metadata";

const locked = new Set<DocumentId>();
const waiters = new Map<DocumentId, Array<() => void>>();

export function tryLockDocument(documentId: DocumentId): boolean {
	if (locked.has(documentId)) {
		return false;
	}

	locked.add(documentId);
	return true;
}

export function waitForDocumentLock(documentId: DocumentId): Promise<void> {
	if (tryLockDocument(documentId)) {
		return Promise.resolve();
	}

	return new Promise((resolve) => {
		if (!waiters.has(documentId)) {
			waiters.set(documentId, []);
		}

		waiters.get(documentId)!.push(resolve);
	});
}

export function unlockDocument(documentId: DocumentId): void {
	if (!locked.has(documentId)) {
		throw new Error(`Document ${documentId} is not locked`);
	}

	if (waiters.has(documentId)) {
		waiters.get(documentId)!.shift()?.();
	} else {
		locked.delete(documentId);
	}
}
