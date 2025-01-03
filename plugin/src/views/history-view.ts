import type { IconName, WorkspaceLeaf } from "obsidian";
import { ItemView, setIcon } from "obsidian";
import type { HistoryEntry, SyncHistory } from "src/tracing/sync-history";
import { SyncType } from "src/tracing/sync-history";
import { SyncSource, SyncStatus } from "src/tracing/sync-history";
import { intlFormatDistance } from "date-fns";
import type { Database } from "src/database/database";

export class HistoryView extends ItemView {
	public static readonly TYPE = "history-view";
	public static readonly ICON = "square-stack";
	private timer: NodeJS.Timer | null = null;

	public constructor(
		leaf: WorkspaceLeaf,
		private readonly database: Database,
		private readonly history: SyncHistory
	) {
		super(leaf);
		this.icon = HistoryView.ICON;

		// eslint-disable-next-line @typescript-eslint/no-misused-promises
		history.addSyncHistoryUpdateListener(async () => {
			await this.updateView();
		});
	}

	private static getSyncTypeIcon(type: SyncType | undefined): IconName {
		switch (type) {
			case SyncType.CREATE:
				return "file-plus";
			case SyncType.DELETE:
				return "trash-2";
			case SyncType.UPDATE:
				return "file-pen-line";
			case undefined:
			default:
				return "";
		}
	}

	private static getSyncSourceIcon(source: SyncSource | undefined): IconName {
		switch (source) {
			case SyncSource.PUSH:
				return "upload";
			case SyncSource.PULL:
				return "download";
			case undefined:
			default:
				return "";
		}
	}

	private static renderSyncItemTitle(
		element: HTMLElement,
		entry: HistoryEntry
	): void {
		const syncTypeIcon = HistoryView.getSyncTypeIcon(entry.type);
		if (syncTypeIcon) {
			setIcon(element.createDiv(), syncTypeIcon);
		}

		element.createEl("span", {
			text: entry.relativePath,
		});

		const syncSourceIcon = HistoryView.getSyncSourceIcon(entry.source);
		if (syncSourceIcon) {
			setIcon(element.createDiv(), syncSourceIcon);
		}
	}

	public getViewType(): string {
		return HistoryView.TYPE;
	}

	public getDisplayText(): string {
		return "VaultLink history";
	}

	public async onOpen(): Promise<void> {
		await this.updateView();
		// eslint-disable-next-line @typescript-eslint/no-misused-promises
		this.timer = setInterval(async () => this.updateView(), 1000);
	}

	public async onClose(): Promise<void> {
		if (this.timer) {
			clearInterval(this.timer);
		}
	}

	private async updateView(): Promise<void> {
		const container = this.containerEl.children[1];
		container.empty();
		container.createEl("h4", { text: "VaultLink History" });

		const entries = this.history
			.getEntries()
			.reverse()
			.filter(
				(entry) =>
					entry.status !== SyncStatus.NO_OP ||
					this.database.getSettings().displayNoopSyncEvents
			);

		entries.forEach((entry) => {
			container.createDiv(
				{
					cls: ["history-card", entry.status.toLocaleLowerCase()],
				},
				(card) => {
					if (
						this.app.vault.getFileByPath(entry.relativePath) !==
						null
					) {
						card.addEventListener("click", () => {
							void this.app.workspace.openLinkText(
								entry.relativePath,
								entry.relativePath,
								false
							);
						});

						card.addClass("clickable");
					}

					card.createDiv(
						{
							cls: "history-card-header",
						},
						(header) => {
							header.createEl(
								"h5",
								{
									cls: "history-card-title",
								},
								(title) => {
									HistoryView.renderSyncItemTitle(
										title,
										entry
									);
								}
							);

							header.createSpan({
								text: intlFormatDistance(
									entry.timestamp,
									new Date()
								),
								cls: "history-card-timestamp",
							});
						}
					);

					card.createEl("p", {
						text: `${entry.message}.`,
						cls: "history-card-message",
					});
				}
			);
		});
	}
}
