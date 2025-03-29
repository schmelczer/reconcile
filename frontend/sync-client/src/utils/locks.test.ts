import { Logger } from "../tracing/logger";
import type { RelativePath } from "../persistence/database";
import { Locks } from "./locks";

describe("Document lock", () => {
	const testPath: RelativePath = "test/document/path";
	const logger = new Logger();

	// eslint-disable-next-line @typescript-eslint/init-declarations
	let locks: Locks<RelativePath>;

	beforeEach(() => {
		locks = new Locks<RelativePath>(logger);
	});

	test("should lock a document successfully", () => {
		const result = locks.tryLock(testPath);
		expect(result).toBe(true);
	});

	test("should not lock a document that is already locked", () => {
		locks.tryLock(testPath);
		const result = locks.tryLock(testPath);
		expect(result).toBe(false);
	});

	test("should unlock a locked document", () => {
		locks.tryLock(testPath);
		locks.unlock(testPath);
		const result = locks.tryLock(testPath);
		expect(result).toBe(true);
		locks.unlock(testPath);
	});

	test("should throw an error when unlocking a document that is not locked", () => {
		expect(() => {
			locks.unlock(testPath);
		}).toThrow(`Document ${testPath} is not locked, cannot unlock`);
	});

	test("should wait for a document lock and resolve when unlocked", async () => {
		locks.tryLock(testPath);

		let resolved = false;
		const waitPromise = locks.waitForLock(testPath).then(() => {
			resolved = true;
		});

		locks.unlock(testPath);
		await waitPromise;

		expect(resolved).toBe(true);
	});

	test("should resolve multiple waiters in FIFO order", async () => {
		locks.tryLock(testPath);

		let firstResolved = false;
		let secondResolved = false;
		let thirdResolved = false;

		const firstWaitPromise = locks.waitForLock(testPath).then(() => {
			firstResolved = true;
		});

		const secondWaitPromise = locks.waitForLock(testPath).then(() => {
			secondResolved = true;
		});

		const thirdWaitPromise = locks.waitForLock(testPath).then(() => {
			thirdResolved = true;
		});

		locks.unlock(testPath);
		await firstWaitPromise;
		expect(firstResolved).toBe(true);
		expect(secondResolved).toBe(false);
		expect(thirdResolved).toBe(false);

		locks.unlock(testPath);
		await secondWaitPromise;
		expect(secondResolved).toBe(true);
		expect(thirdResolved).toBe(false);

		locks.unlock(testPath);
		await thirdWaitPromise;
		expect(thirdResolved).toBe(true);
	});
});
