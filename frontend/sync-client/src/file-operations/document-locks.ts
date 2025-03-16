import type { Logger } from "../tracing/logger";
import type { RelativePath } from "../persistence/database";

// Manages locks on documents to prevent concurrent modifications
// allowing the client's FileOperations implementation to be simpler.
// Locks are granted in a first-in-first-out order.
export class DocumentLocks {
	private readonly locked = new Set<RelativePath>();
	private readonly waiters = new Map<RelativePath, (() => void)[]>();

	public constructor(private readonly logger: Logger) {}

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

		this.logger.debug(`Waiting for lock on ${relativePath}`);

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
			this.logger.debug(`Granted lock on ${relativePath}`);
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
