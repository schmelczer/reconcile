import type { RelativePath } from "src/database/document-metadata";

const locked = new Set<RelativePath>();
const waiters = new Map<RelativePath, (() => void)[]>();

export function tryLockDocument(relativePath: RelativePath): boolean {
	if (locked.has(relativePath)) {
		return false;
	}

	locked.add(relativePath);
	return true;
}

export async function waitForDocumentLock(
	relativePath: RelativePath
): Promise<void> {
	if (tryLockDocument(relativePath)) {
		return Promise.resolve();
	}

	return new Promise((resolve) => {
		let waiting = waiters.get(relativePath);
		if (!waiting) {
			waiting = [];
			waiters.set(relativePath, waiting);
		}

		waiting.push(resolve);
	});
}

export function unlockDocument(relativePath: RelativePath): void {
	if (!locked.has(relativePath)) {
		throw new Error(
			`Document ${relativePath} is not locked, cannot unlock`
		);
	}

	// Remove the first element to ensure FIFO unblocking order
	const nextWaiting = waiters.get(relativePath)?.shift();

	if (nextWaiting) {
		nextWaiting();
	} else {
		locked.delete(relativePath);
	}
}
