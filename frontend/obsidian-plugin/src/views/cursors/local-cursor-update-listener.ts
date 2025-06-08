import type { Workspace } from "obsidian";
import { EventRef, Editor, MarkdownView, MarkdownFileInfo } from "obsidian";
import type { Logger, SyncClient } from "sync-client";
import type { Cursor } from "./get-cursors-from-editor";
import { getCursorsFromEditor } from "./get-cursors-from-editor";

export class LocalCursorUpdateListener {
	private static readonly UPDATE_INTERVAL_MS = 50;
	private readonly eventHandle: NodeJS.Timeout;
	private lastCursorState: Record<string, Cursor[]> = {};

	public constructor(
		private readonly client: SyncClient,
		private readonly workspace: Workspace
	) {
		this.eventHandle = setInterval(() => {
			this.updateAllCursors();
		}, LocalCursorUpdateListener.UPDATE_INTERVAL_MS);
	}

	public dispose(): void {
		clearInterval(this.eventHandle);
	}

	private updateAllCursors(): void {
		const currentCursors = this.getAllCursors();
		if (
			JSON.stringify(this.lastCursorState) ===
			JSON.stringify(currentCursors)
		) {
			return;
		}
		this.lastCursorState = currentCursors;
		this.client
			.updateLocalCursors(currentCursors)
			.catch((error: unknown) => {
				this.client.logger.error(
					`Failed to update local cursors: ${error}`
				);
			});
	}

	private getAllCursors(): Record<string, Cursor[]> {
		const cursors: Record<string, Cursor[]> = {};
		this.workspace
			.getLeavesOfType("markdown")
			.map((leaf) => leaf.view)
			.filter((view) => view instanceof MarkdownView)
			.forEach((view) => {
				const { file } = view;
				if (!file) {
					return;
				}
				cursors[file.path] = getCursorsFromEditor(view.editor);
			});
		return cursors;
	}
}
