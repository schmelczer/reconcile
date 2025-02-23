import { FileSystemOperations } from "sync-client";
import type { RelativePath } from "../persistence/database";
import { FileOperations } from "./file-operations";
import { Logger } from "../tracing/logger";
import assert from "assert";

describe("File operations", () => {
	class FakeFileSystemOperations implements FileSystemOperations {
		public readonly names = new Set<string>();

		public listAllFiles(): Promise<RelativePath[]> {
			throw new Error("Method not implemented.");
		}
		public read(path: RelativePath): Promise<Uint8Array> {
			throw new Error("Method not implemented.");
		}
		public async write(
			path: RelativePath,
			_content: Uint8Array
		): Promise<void> {
			this.names.add(path);
		}
		public atomicUpdateText(
			path: RelativePath,
			updater: (currentContent: string) => string
		): Promise<string> {
			throw new Error("Method not implemented.");
		}
		public getFileSize(path: RelativePath): Promise<number> {
			throw new Error("Method not implemented.");
		}
		public getModificationTime(path: RelativePath): Promise<Date> {
			throw new Error("Method not implemented.");
		}
		public async exists(path: RelativePath): Promise<boolean> {
			return this.names.has(path);
		}
		public async createDirectory(path: RelativePath): Promise<void> {}
		public delete(path: RelativePath): Promise<void> {
			throw new Error("Method not implemented.");
		}
		public async rename(
			oldPath: RelativePath,
			newPath: RelativePath
		): Promise<void> {
			this.names.delete(oldPath);
			this.names.add(newPath);
		}
	}

	test("should deconflict renames", async () => {
		let fs = new FakeFileSystemOperations();
		let fileOperations = new FileOperations(new Logger(), fs);

		await fileOperations.create("a", new Uint8Array());
		assertSetOnlyContains(fs.names, "a");
		await fileOperations.move("a", "b");
		assertSetOnlyContains(fs.names, "b");

		await fileOperations.create("c", new Uint8Array());
		assertSetOnlyContains(fs.names, "b", "c");

		await fileOperations.move("c", "b");
		assertSetOnlyContains(fs.names, "b", "b (1)");

		await fileOperations.create("c", new Uint8Array());
		await fileOperations.move("c", "b");
		assertSetOnlyContains(fs.names, "b", "b (1)", "b (2)");
	});

	test("should deconflict renames with file extension", async () => {
		let fs = new FakeFileSystemOperations();
		let fileOperations = new FileOperations(new Logger(), fs);

		await fileOperations.create("b.md", new Uint8Array());
		await fileOperations.create("c.md", new Uint8Array());
		await fileOperations.move("c.md", "b.md");
		assertSetOnlyContains(fs.names, "b.md", "b (1).md");
	});

	test("should deconflict renames with paths", async () => {
		let fs = new FakeFileSystemOperations();
		let fileOperations = new FileOperations(new Logger(), fs);

		await fileOperations.create("a/b.c/d", new Uint8Array());
		await fileOperations.create("a/b.c/e", new Uint8Array());
		await fileOperations.move("a/b.c/d", "a/b.c/e");
		assertSetOnlyContains(fs.names, "a/b.c/e", "a/b.c/e (1)");
	});
});

function assertSetOnlyContains<T>(set: Set<T>, ...values: T[]) {
	assert(
		set.size === values.length &&
			Array.from(set).every((value) => values.includes(value)),
		`Expected set to contain only ${values.map((v) => '"' + v + '"').join(", ")}, but it contained ${Array.from(
			set
		)
			.map((v) => '"' + v + '"')
			.join(", ")}`
	);
}
