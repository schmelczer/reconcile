import type { RelativePath } from "../persistence/database";
import { DocumentLocks } from "./document-locks";

describe("Document lock", () => {
	const testPath: RelativePath = "test/document/path";
	let locks = new DocumentLocks();

	beforeEach(() => {
		locks = new DocumentLocks();
	});

	test("should lock a document successfully", () => {
		const result = locks.tryLockDocument(testPath);
		expect(result).toBe(true);
	});

	test("should not lock a document that is already locked", () => {
		locks.tryLockDocument(testPath);
		const result = locks.tryLockDocument(testPath);
		expect(result).toBe(false);
	});

	test("should unlock a locked document", () => {
		locks.tryLockDocument(testPath);
		locks.unlockDocument(testPath);
		const result = locks.tryLockDocument(testPath);
		expect(result).toBe(true);
		locks.unlockDocument(testPath);
	});

	test("should throw an error when unlocking a document that is not locked", () => {
		expect(() => {
			locks.unlockDocument(testPath);
		}).toThrow(`Document ${testPath} is not locked, cannot unlock`);
	});

	test("should wait for a document lock and resolve when unlocked", async () => {
		locks.tryLockDocument(testPath);

		let resolved = false;
		const waitPromise = locks.waitForDocumentLock(testPath).then(() => {
			resolved = true;
		});

		locks.unlockDocument(testPath);
		await waitPromise;

		expect(resolved).toBe(true);
	});

	test("should resolve multiple waiters in FIFO order", async () => {
		locks.tryLockDocument(testPath);

		let firstResolved = false;
		let secondResolved = false;
		let thirdResolved = false;

		const firstWaitPromise = locks
			.waitForDocumentLock(testPath)
			.then(() => {
				firstResolved = true;
			});

		const secondWaitPromise = locks
			.waitForDocumentLock(testPath)
			.then(() => {
				secondResolved = true;
			});

		const thirdWaitPromise = locks
			.waitForDocumentLock(testPath)
			.then(() => {
				thirdResolved = true;
			});

		locks.unlockDocument(testPath);
		await firstWaitPromise;
		expect(firstResolved).toBe(true);
		expect(secondResolved).toBe(false);
		expect(thirdResolved).toBe(false);

		locks.unlockDocument(testPath);
		await secondWaitPromise;
		expect(secondResolved).toBe(true);
		expect(thirdResolved).toBe(false);

		locks.unlockDocument(testPath);
		await thirdWaitPromise;
		expect(thirdResolved).toBe(true);
	});
});
