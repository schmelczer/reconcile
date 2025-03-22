import type {
	Database,
	DocumentRecord,
	RelativePath
} from "../persistence/database";
import { FileOperations } from "./file-operations";
import { Logger } from "../tracing/logger";
import { assertSetContainsExactly } from "../utils/assert-set-contains-exactly";
import type { FileSystemOperations } from "./filesystem-operations";
import init, { base64ToBytes } from "sync_lib";
import fs from "fs";

class MockDatabase implements Partial<Database> {
	public getLatestDocumentByRelativePath(
		_find: RelativePath
	): DocumentRecord | undefined {
		// no-op
		return undefined;
	}

	public move(
		_oldRelativePath: RelativePath,
		_newRelativePath: RelativePath
	): void {
		// no-op
	}
}

class FakeFileSystemOperations implements FileSystemOperations {
	public readonly names = new Set<string>();

	public async listAllFiles(): Promise<RelativePath[]> {
		throw new Error("Method not implemented.");
	}
	public async read(_path: RelativePath): Promise<Uint8Array> {
		throw new Error("Method not implemented.");
	}
	public async write(
		path: RelativePath,
		_content: Uint8Array
	): Promise<void> {
		this.names.add(path);
	}
	public async atomicUpdateText(
		_path: RelativePath,
		_updater: (currentContent: string) => string
	): Promise<string> {
		throw new Error("Method not implemented.");
	}
	public async getFileSize(_path: RelativePath): Promise<number> {
		throw new Error("Method not implemented.");
	}
	public async getModificationTime(_path: RelativePath): Promise<Date> {
		throw new Error("Method not implemented.");
	}
	public async exists(path: RelativePath): Promise<boolean> {
		return this.names.has(path);
	}
	public async createDirectory(_path: RelativePath): Promise<void> {
		// this is called but irrelevant for this mock
	}
	public async delete(_path: RelativePath): Promise<void> {
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

describe("File operations", () => {
	beforeEach(async () => {
		const wasmBin = fs.readFileSync(
			"../../backend/sync_lib/pkg/sync_lib_bg.wasm"
		);
		await init({ module_or_path: wasmBin });
	});

	it("should deconflict renames", async () => {
		const fileSystemOperations = new FakeFileSystemOperations();
		const fileOperations = new FileOperations(
			new Logger(),
			new MockDatabase() as Database, // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
			fileSystemOperations
		);

		await fileOperations.create("a", new Uint8Array());
		assertSetContainsExactly(fileSystemOperations.names, "a");
		await fileOperations.move("a", "b");
		assertSetContainsExactly(fileSystemOperations.names, "b");

		await fileOperations.create("c", new Uint8Array());
		assertSetContainsExactly(fileSystemOperations.names, "b", "c");

		await fileOperations.move("c", "b");
		assertSetContainsExactly(fileSystemOperations.names, "b", "b (1)");

		await fileOperations.create("c", new Uint8Array());
		await fileOperations.move("c", "b");
		assertSetContainsExactly(
			fileSystemOperations.names,
			"b",
			"b (1)",
			"b (2)"
		);
	});

	it("should deconflict renames with file extension", async () => {
		const fs = new FakeFileSystemOperations();
		const fileOperations = new FileOperations(
			new Logger(),
			new MockDatabase() as Database, // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
			fs
		);

		await fileOperations.create("b.md", new Uint8Array());
		await fileOperations.create("c.md", new Uint8Array());
		await fileOperations.move("c.md", "b.md");
		assertSetContainsExactly(fs.names, "b.md", "b (1).md");

		await fileOperations.create("d.md", new Uint8Array());
		await fileOperations.move("d.md", "b.md");
		assertSetContainsExactly(fs.names, "b.md", "b (1).md", "b (2).md");

		await fileOperations.create("file-23.md", new Uint8Array());
		await fileOperations.create("file-23 (1).md", new Uint8Array());
		await fileOperations.move("file-23.md", "file-23 (1).md");
		assertSetContainsExactly(
			fs.names,
			"b.md",
			"b (1).md",
			"b (2).md",
			"file-23 (1).md",
			"file-23 (2).md"
		);
	});

	it("should deconflict renames with paths", async () => {
		const fs = new FakeFileSystemOperations();
		const fileOperations = new FileOperations(
			new Logger(),
			new MockDatabase() as Database, // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
			fs
		);

		await fileOperations.create("a/b.c/d", new Uint8Array());
		await fileOperations.create("a/b.c/e", new Uint8Array());
		await fileOperations.move("a/b.c/d", "a/b.c/e");
		assertSetContainsExactly(fs.names, "a/b.c/e", "a/b.c/e (1)");
	});
});
