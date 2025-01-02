import type { WorkspaceLeaf } from "obsidian";
import { ItemView } from "obsidian";
import type { SyncHistory } from "src/tracing/sync-history";
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

	private static formatSource(source: SyncSource | undefined): string {
		switch (source) {
			case SyncSource.PUSH:
				return " ⤴️";
			case SyncSource.PULL:
				return " ⤵️";
			case undefined:
			default:
				return "";
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

		this.history
			.getEntries()
			.reverse()
			.filter(
				(entry) =>
					entry.status !== SyncStatus.NO_OP ||
					this.database.getSettings().displayNoopSyncEvents
			)
			.forEach((entry) => {
				const card = container.createDiv({
					cls: ["history-card", entry.status.toLocaleLowerCase()],
				});
				const header = card.createDiv({ cls: "history-card-header" });
				header.createEl("h5", {
					text:
						entry.relativePath +
						HistoryView.formatSource(entry.source),
					cls: "history-card-title",
				});
				header.createSpan({
					text: intlFormatDistance(entry.timestamp, new Date()),
					cls: "history-card-timestamp",
				});
				card.createEl("p", {
					text: entry.message,
					cls: "history-card-message",
				});
			});
	}
}
