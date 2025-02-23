import type { RelativePath } from "../persistence/database";

export class DocumentLocks {
	private readonly locked = new Set<RelativePath>();
	private readonly waiters = new Map<RelativePath, (() => void)[]>();

	public tryLockDocument(relativePath: RelativePath): boolean {
		if (this.locked.has(relativePath)) {
			return false;
		}

		this.locked.add(relativePath);
		return true;
	}

	public async waitForDocumentLock(
		relativePath: RelativePath
	): Promise<void> {
		if (this.tryLockDocument(relativePath)) {
			return Promise.resolve();
		}

		return new Promise((resolve) => {
			let waiting = this.waiters.get(relativePath);
			if (!waiting) {
				waiting = [];
				this.waiters.set(relativePath, waiting);
			}

			waiting.push(resolve);
		});
	}

	public unlockDocument(relativePath: RelativePath): void {
		if (!this.locked.has(relativePath)) {
			throw new Error(
				`Document ${relativePath} is not locked, cannot unlock`
			);
		}

		// Remove the first element to ensure FIFO unblocking order
		const nextWaiting = this.waiters.get(relativePath)?.shift();

		if (nextWaiting) {
			nextWaiting();
		} else {
			this.locked.delete(relativePath);
		}
	}

	public reset(): void {
		this.locked.clear();
		this.waiters.clear();
	}
}
