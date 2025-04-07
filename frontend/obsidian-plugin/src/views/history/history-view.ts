import "./history-view.scss";

import type { IconName, WorkspaceLeaf } from "obsidian";
import { ItemView, setIcon } from "obsidian";
import { intlFormatDistance } from "date-fns";
import type { HistoryEntry, SyncClient } from "sync-client";
import { SyncType } from "sync-client";

export class HistoryView extends ItemView {
	public static readonly TYPE = "history-view";
	public static readonly ICON = "square-stack";
	private timer: NodeJS.Timeout | null = null;

	private historyContainer: HTMLElement | undefined;
	private readonly historyEntryToElement = new Map<
		HistoryEntry,
		HTMLElement
	>();

	public constructor(
		leaf: WorkspaceLeaf,
		private readonly client: SyncClient
	) {
		super(leaf);
		this.icon = HistoryView.ICON;

		this.client.addSyncHistoryUpdateListener(
			() =>
				void this.updateView().catch((error: unknown) => {
					this.client.logger.error(
						`Failed to update history view: ${error}`
					);
				})
		);
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

	private static renderSyncItemTitle(
		element: HTMLElement,
		entry: HistoryEntry
	): void {
		const syncTypeIcon = HistoryView.getSyncTypeIcon(entry.type);
		if (syncTypeIcon) {
			setIcon(element.createDiv(), syncTypeIcon);
		}

		element.createEl("span", {
			text: entry.relativePath.split("/").pop()
		});
	}

	private static updateTimeSince(
		element: HTMLElement,
		entry: HistoryEntry
	): void {
		const timestampElement = element.querySelector(
			".history-card-timestamp"
		);
		if (timestampElement != null) {
			timestampElement.textContent = intlFormatDistance(
				entry.timestamp,
				new Date()
			);
		}
	}

	public getViewType(): string {
		return HistoryView.TYPE;
	}

	public getDisplayText(): string {
		return "VaultLink history";
	}

	public async onOpen(): Promise<void> {
		const container = this.containerEl.children[1];
		container.createEl("h4", { text: "VaultLink history" });

		this.historyContainer = container.createDiv({ cls: "logs-container" });

		await this.updateView();
		this.timer = setInterval(() => void this.updateView(), 1000);
	}

	public async onClose(): Promise<void> {
		if (this.timer) {
			clearInterval(this.timer);
		}
	}

	private async updateView(): Promise<void> {
		const container = this.historyContainer;
		if (container === undefined) {
			return;
		}

		// entries are newest first, but we prepend new ones
		const entries = this.client.getHistoryEntries().toReversed();

		if (this.historyEntryToElement.size === 0 && entries.length > 0) {
			// Clear the "No update has happened yet" message
			container.empty();
		}

		entries.forEach((entry) => {
			const element = this.historyEntryToElement.get(entry);
			if (element !== undefined) {
				HistoryView.updateTimeSince(element, entry);
				return;
			}

			const newElement = this.createHistoryCard(container, entry);
			container.prepend(newElement);
			this.historyEntryToElement.set(entry, newElement);
		});

		const newEntries = new Set(entries);
		for (const [entry, element] of this.historyEntryToElement) {
			if (!newEntries.has(entry)) {
				element.remove();
				this.historyEntryToElement.delete(entry);
			}
		}

		if (entries.length === 0) {
			container.empty();
			container.createEl("p", {
				text: "No update has happened yet."
			});
		}
	}

	private createHistoryCard(
		container: HTMLElement,
		entry: HistoryEntry
	): HTMLElement {
		return container.createDiv(
			{
				cls: ["history-card", entry.status.toLocaleLowerCase()]
			},
			(card) => {
				if (this.app.vault.getFileByPath(entry.relativePath) !== null) {
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
						cls: "history-card-header"
					},
					(header) => {
						header.createEl(
							"h5",
							{
								cls: "history-card-title"
							},
							(title) => {
								HistoryView.renderSyncItemTitle(title, entry);
							}
						);

						header.createSpan({
							text: intlFormatDistance(
								entry.timestamp,
								new Date()
							),
							cls: "history-card-timestamp"
						});
					}
				);

				card.createEl("p", {
					text: `${entry.message}.`,
					cls: "history-card-message"
				});
			}
		);
	}
}
