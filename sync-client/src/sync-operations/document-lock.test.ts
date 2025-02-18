import {
	tryLockDocument,
	waitForDocumentLock,
	unlockDocument
} from "./document-lock";
import type { RelativePath } from "src/database/document-metadata";

describe("Document Lock Operations", () => {
	const testPath: RelativePath = "test/document/path";

	beforeEach(() => {
		// Reset the state before each test
		(global as any).locked = new Set<RelativePath>();
		(global as any).waiters = new Map<RelativePath, (() => void)[]>();
	});

	test("should lock a document successfully", () => {
		const result = tryLockDocument(testPath);
		expect(result).toBe(true);
	});

	test("should not lock a document that is already locked", () => {
		tryLockDocument(testPath);
		const result = tryLockDocument(testPath);
		expect(result).toBe(false);
	});

	test("should unlock a locked document", () => {
		tryLockDocument(testPath);
		unlockDocument(testPath);
		const result = tryLockDocument(testPath);
		expect(result).toBe(true);
		unlockDocument(testPath);
	});

	test("should throw an error when unlocking a document that is not locked", () => {
		expect(() => {
			unlockDocument(testPath);
		}).toThrow(`Document ${testPath} is not locked, cannot unlock`);
	});

	test("should wait for a document lock and resolve when unlocked", async () => {
		tryLockDocument(testPath);

		let resolved = false;
		const waitPromise = waitForDocumentLock(testPath).then(() => {
			resolved = true;
		});

		unlockDocument(testPath);
		await waitPromise;

		expect(resolved).toBe(true);
	});

	test("should resolve multiple waiters in FIFO order", async () => {
		tryLockDocument(testPath);

		let firstResolved = false;
		let secondResolved = false;

		const firstWaitPromise = waitForDocumentLock(testPath).then(() => {
			firstResolved = true;
		});

		const secondWaitPromise = waitForDocumentLock(testPath).then(() => {
			secondResolved = true;
		});

		unlockDocument(testPath);
		await firstWaitPromise;
		expect(firstResolved).toBe(true);
		expect(secondResolved).toBe(false);

		unlockDocument(testPath);
		await secondWaitPromise;
		expect(secondResolved).toBe(true);
	});
});
