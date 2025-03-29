import type { Logger } from "../tracing/logger";

// Manages locks on T to prevent concurrent modifications
// allowing the client's FileOperations implementation to be simpler.
// Locks are granted in a first-in-first-out order.
export class Locks<T> {
	private readonly locked = new Set<T>();
	private readonly waiters = new Map<T, (() => void)[]>();

	public constructor(private readonly logger: Logger) {}

	public tryLock(key: T): boolean {
		if (this.locked.has(key)) {
			return false;
		}

		this.locked.add(key);

		return true;
	}

	public async waitForLock(key: T): Promise<void> {
		if (this.tryLock(key)) {
			return Promise.resolve();
		}

		this.logger.debug(`Waiting for lock on ${key}`);

		return new Promise((resolve) => {
			let waiting = this.waiters.get(key);
			if (!waiting) {
				waiting = [];
				this.waiters.set(key, waiting);
			}

			waiting.push(resolve);
		});
	}

	public unlock(key: T): void {
		if (!this.locked.has(key)) {
			throw new Error(`Document ${key} is not locked, cannot unlock`);
		}

		// Remove the first element to ensure FIFO unblocking order
		const nextWaiting = this.waiters.get(key)?.shift();

		if (nextWaiting) {
			this.logger.debug(`Granted lock on ${key}`);
			nextWaiting();
		} else {
			this.locked.delete(key);
		}
	}

	public reset(): void {
		this.locked.clear();
		this.waiters.clear();
	}
}
