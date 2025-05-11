import type { Workspace } from "obsidian";
import { FileView, setIcon } from "obsidian";
import type { SyncClient } from "sync-client";
import { DocumentUpdateStatus } from "sync-client";
import "./editor-sync-line.scss";

export function updateEditorStatusDisplay(
	workspace: Workspace,
	client: SyncClient
): void {
	workspace.iterateAllLeaves((leaf) => {
		if (leaf.view instanceof FileView) {
			const filePath = leaf.view.file?.path;
			if (filePath == null) {
				return;
			}
			const parent = leaf.view.contentEl.querySelector(".cm-editor");
			if (parent == null) {
				return;
			}

			const element =
				parent.querySelector(".vault-link-sync-status") ??
				parent.createDiv(
					{
						cls: "vault-link-sync-status"
					},
					(el) => {
						el.createSpan({ text: "VaultLink sync state" });
						el.createDiv({
							cls: "icon"
						});
					}
				);

			const isLoading =
				client.getDocumentSyncingStatus(filePath) ==
				DocumentUpdateStatus.SYNCING;

			if (isLoading) {
				element.classList.add("loading");
			} else {
				element.classList.remove("loading");
			}

			const iconContainer = element.querySelector(".icon");
			if (iconContainer != null) {
				setIcon(
					iconContainer as HTMLElement, // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
					isLoading ? "loader" : "circle-check"
				);
			}
		}
	});
}
