import type { Stat, Vault, Workspace } from "obsidian";
import { MarkdownView, normalizePath } from "obsidian";
import type {
	FileSystemOperations,
	RelativePath,
	TextWithCursors
} from "sync-client";
import { lineAndColumnToPosition } from "./utils/line-and-column-to-position";
import { positionToLineAndColumn } from "./utils/position-to-line-and-column";

export class ObsidianFileSystemOperations implements FileSystemOperations {
	public constructor(
		private readonly vault: Vault,
		private readonly workspace: Workspace
	) {}

	public async listAllFiles(): Promise<RelativePath[]> {
		// Let's implement this by hand because vault.adapter.listAllFiles doesn't always return all files.
		const allFiles = [];
		const remainingFolders = [this.vault.getRoot().path];

		// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
		while (true) {
			const folder = remainingFolders.pop();
			if (folder == undefined) {
				break;
			}

			if (folder.includes(".obsidian")) {
				continue;
			}

			const files = await this.vault.adapter.list(normalizePath(folder));
			allFiles.push(...files.files);
			remainingFolders.push(...files.folders);
		}

		return allFiles;
	}

	public async read(path: RelativePath): Promise<Uint8Array> {
		path = normalizePath(path);
		const view = this.workspace.getActiveViewOfType(MarkdownView);
		if (view?.file?.path === path) {
			return new TextEncoder().encode(view.editor.getValue());
		}

		return new Uint8Array(await this.vault.adapter.readBinary(path));
	}

	public async write(path: RelativePath, content: Uint8Array): Promise<void> {
		path = normalizePath(path);

		const view = this.workspace.getActiveViewOfType(MarkdownView);
		if (view?.file?.path === path) {
			const position = view.editor.getCursor();
			view.editor.setValue(new TextDecoder().decode(content));
			view.editor.setCursor(position);
			return;
		}

		return this.vault.adapter.writeBinary(
			path,
			// eslint-disable-next-line @typescript-eslint/no-unsafe-type-assertion
			content.buffer as ArrayBuffer
		);
	}

	public async atomicUpdateText(
		path: RelativePath,
		updater: (current: TextWithCursors) => TextWithCursors
	): Promise<string> {
		path = normalizePath(path);

		const view = this.workspace.getActiveViewOfType(MarkdownView);

		if (view?.file?.path === path) {
			const text = view.editor.getValue();
			const cursors = view.editor
				.listSelections()
				.flatMap(({ anchor, head }, i) => [
					{
						id: 2 * i,
						characterPosition: lineAndColumnToPosition(
							text,
							anchor.line,
							anchor.ch
						)
					},
					{
						id: 2 * i + 1,
						characterPosition: lineAndColumnToPosition(
							text,
							head.line,
							head.ch
						)
					}
				]);

			const result = updater({
				text,
				cursors
			});

			if (result.text === text) {
				return text;
			}

			view.editor.setValue(result.text);

			const selections = [];
			for (let i = 0; i < result.cursors.length / 2; i++) {
				const from = result.cursors[2 * i];
				const to = result.cursors[2 * i + 1];
				const { line: fromLine, column: fromColumn } =
					positionToLineAndColumn(
						result.text,
						from.characterPosition
					);

				const { line: toLine, column: toColumn } =
					positionToLineAndColumn(result.text, to.characterPosition);

				selections.push({
					anchor: { line: fromLine, ch: fromColumn },
					head: { line: toLine, ch: toColumn }
				});
			}
			view.editor.setSelections(selections);

			return result.text;
		}

		return this.vault.adapter.process(
			path,
			(text) =>
				updater({
					text,
					cursors: []
				}).text
		);
	}

	public async getFileSize(path: RelativePath): Promise<number> {
		return (await this.statFile(path)).size;
	}

	public async getModificationTime(path: RelativePath): Promise<Date> {
		return new Date((await this.statFile(path)).mtime);
	}

	public async exists(path: RelativePath): Promise<boolean> {
		return this.vault.adapter.exists(normalizePath(path));
	}

	public async createDirectory(path: RelativePath): Promise<void> {
		return this.vault.adapter.mkdir(normalizePath(path));
	}

	public async delete(path: RelativePath): Promise<void> {
		if (!(await this.vault.adapter.trashSystem(normalizePath(path)))) {
			return this.vault.adapter.remove(normalizePath(path));
		}
	}

	public async rename(
		oldPath: RelativePath,
		newPath: RelativePath
	): Promise<void> {
		return this.vault.adapter.rename(oldPath, newPath);
	}

	private async statFile(path: string): Promise<Stat> {
		const file = await this.vault.adapter.stat(normalizePath(path));

		if (!file) {
			throw new Error(`File not found: ${path}`);
		}

		return file;
	}
}
