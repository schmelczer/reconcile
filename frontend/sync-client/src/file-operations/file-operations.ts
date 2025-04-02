import type { Logger } from "../tracing/logger";
import type {
	FileSystemOperations,
	TextWithCursors
} from "./filesystem-operations";
import type { Database, RelativePath } from "../persistence/database";
import {
	CursorPosition,
	isBinary,
	isFileTypeMergable,
	mergeTextWithCursors,
	TextWithCursors as RustTextWithCursors
} from "sync_lib";
import { SafeFileSystemOperations } from "./safe-filesystem-operations";

export class FileOperations {
	private static readonly PARENTHESES_REGEX = / \((\d+)\)$/;
	private readonly fs: SafeFileSystemOperations;

	public constructor(
		private readonly logger: Logger,
		private readonly database: Database,
		fs: FileSystemOperations,
		private readonly nativeLineEndings = "\n"
	) {
		this.fs = new SafeFileSystemOperations(fs, logger);
	}

	public async listAllFiles(): Promise<RelativePath[]> {
		return this.fs.listAllFiles();
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		return this.fromNativeLineEndings(await this.fs.read(path));
	}

	/**
	 * Create a file at the specified path.
	 *
	 * If a file with the same name already exists, it is moved before creating the new one.
	 * Parent directories are created if necessary.
	 */
	public async create(
		path: RelativePath,
		newContent: Uint8Array
	): Promise<void> {
		await this.ensureClearPath(path);
		return this.fs.write(path, this.toNativeLineEndings(newContent));
	}

	public async ensureClearPath(path: RelativePath): Promise<void> {
		if (await this.fs.exists(path)) {
			const deconflictedPath = await this.deconflictPath(path);
			this.logger.debug(
				`Didn't expect ${path} to exist, deconflicting by moving it to '${deconflictedPath}'`
			);

			this.database.move(path, deconflictedPath);
			await this.fs.rename(path, deconflictedPath);
		} else {
			await this.createParentDirectories(path);
		}
	}

	/**
	 * Update the file at the given path.
	 *
	 * Performs a 3-way merge before writing if the file's content differs from `expectedContent`.
	 * Does not recreate the file if it no longer exists, returning an empty array instead.
	 */
	public async write(
		path: RelativePath,
		expectedContent: Uint8Array,
		newContent: Uint8Array
	): Promise<void> {
		if (!(await this.fs.exists(path))) {
			this.logger.debug(
				`The caller assumed ${path} exists, but it no longer, so we wont recreate it`
			);
			return;
		}

		if (
			!isFileTypeMergable(path) ||
			isBinary(expectedContent) ||
			isBinary(newContent)
		) {
			this.logger.debug(
				`The expected content is not mergable, so we won't perform a 3-way merge, just overwrite it`
			);
			await this.fs.write(
				path,
				// `newContent` might not be binary so we still have to ensure the line endings are correct
				this.toNativeLineEndings(newContent)
			);
			return;
		}

		const expectedText = new TextDecoder().decode(expectedContent); // this comes from a previous read which must only have \n line endings
		const newText = new TextDecoder().decode(newContent); // this comes from the server which stores text with \n line endings

		await this.fs.atomicUpdateText(
			path,
			({ text, cursors }: TextWithCursors): TextWithCursors => {
				text = text.replace(this.nativeLineEndings, "\n");

				this.logger.debug(
					`Performing a 3-way merge for ${path} with the expected content`
				);

				const left = new RustTextWithCursors(
					text,
					cursors.map(
						(cursor) =>
							new CursorPosition(
								cursor.id,
								cursor.characterPosition
							)
					)
				);
				const right = new RustTextWithCursors(newText, []);
				const merged = mergeTextWithCursors(expectedText, left, right);

				const resultText = merged
					.text()
					.replace("\n", this.nativeLineEndings);

				const resultCursors = merged.cursors().map((cursor) => ({
					id: cursor.id(),
					characterPosition: cursor.characterPosition()
				}));

				merged.free();

				return {
					text: resultText,
					cursors: resultCursors
				};
			}
		);
	}

	public async delete(path: RelativePath): Promise<void> {
		if (await this.exists(path)) {
			return this.fs.delete(path);
		} else {
			this.logger.debug(`No need to delete '${path}', it doesn't exist`);
		}
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		return this.fs.getFileSize(path);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return this.fs.exists(path);
	}

	public async move(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		if (oldPath === newPath) {
			return;
		}

		await this.ensureClearPath(newPath);

		this.database.move(oldPath, newPath);
		await this.fs.rename(oldPath, newPath);
	}

	private fromNativeLineEndings(content: Uint8Array): Uint8Array {
		if (isBinary(content)) {
			return content;
		}

		const decoder = new TextDecoder("utf-8");
		let text = decoder.decode(content);
		text = text.replace(this.nativeLineEndings, "\n");
		return new TextEncoder().encode(text);
	}

	private toNativeLineEndings(content: Uint8Array): Uint8Array {
		if (isBinary(content)) {
			return content;
		}

		const decoder = new TextDecoder("utf-8");
		let text = decoder.decode(content);
		text = text.replace("\n", this.nativeLineEndings);
		return new TextEncoder().encode(text);
	}

	private async createParentDirectories(path: string): Promise<void> {
		const components = path.split("/");
		if (components.length === 1) {
			return;
		}
		for (let i = 1; i < components.length; i++) {
			const parentDir = components.slice(0, i).join("/");
			if (!(await this.fs.exists(parentDir))) {
				await this.fs.createDirectory(parentDir);
			}
		}
	}

	private async deconflictPath(path: RelativePath): Promise<RelativePath> {
		const pathParts = path.split("/");
		const fileName = pathParts.pop();
		if (fileName == "" || fileName == null) {
			throw new Error(`Path '${path}' cannot be empty`);
		}

		let directory = pathParts.join("/");
		if (directory) {
			directory += "/";
		}

		const nameParts = fileName.split(".");
		const extension =
			nameParts.length > 1 ? "." + nameParts[nameParts.length - 1] : "";
		let stem = extension ? nameParts.slice(0, -1).join(".") : fileName;
		let currentCount = Number.parseInt(
			FileOperations.PARENTHESES_REGEX.exec(stem)?.groups?.[0] ?? "0"
		);
		stem = stem.replace(FileOperations.PARENTHESES_REGEX, "");

		let newName = path;
		do {
			currentCount++;
			newName = `${directory}${stem} (${currentCount})${extension}`;
		} while (await this.fs.exists(newName));

		return newName;
	}
}
