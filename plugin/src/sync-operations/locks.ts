import { RelativePath } from "src/database/document-metadata";

const locked = new Set<RelativePath>();
const waiters = new Map<RelativePath, Array<() => void>>();

export function tryLockDocument(relativePath: RelativePath): boolean {
	if (locked.has(relativePath)) {
		return false;
	}

	locked.add(relativePath);
	return true;
}

export function waitForDocumentLock(relativePath: RelativePath): Promise<void> {
	if (tryLockDocument(relativePath)) {
		return Promise.resolve();
	}

	return new Promise((resolve) => {
		if (!waiters.has(relativePath)) {
			waiters.set(relativePath, []);
		}

		waiters.get(relativePath)!.push(resolve);
	});
}

export function unlockDocument(relativePath: RelativePath): void {
	if (!locked.has(relativePath)) {
		throw new Error(`Document ${relativePath} is not locked`);
	}

	const nextWaiting = waiters.get(relativePath)?.shift();
	if (nextWaiting) {
		nextWaiting();
	} else {
		locked.delete(relativePath);
	}
}
