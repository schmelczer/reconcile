import type { Database } from "src/database/database";
import type VaultLinkPlugin from "src/vault-link-plugin";
import type { Syncer } from "src/sync-operations/syncer";
import type { HistoryStats, SyncHistory } from "src/tracing/sync-history";

export class StatusBar {
	private readonly statusBarItem: HTMLElement;

	private lastHistoryStats: HistoryStats | undefined;
	private lastRemaining: number | undefined;

	public constructor(
		private readonly database: Database,
		private readonly plugin: VaultLinkPlugin,
		history: SyncHistory,
		syncer: Syncer
	) {
		this.statusBarItem = plugin.addStatusBarItem();
		history.addSyncHistoryUpdateListener((status) => {
			this.lastHistoryStats = status;
			this.updateStatus();
		});

		syncer.addRemainingOperationsListener((remainingOperations) => {
			this.lastRemaining = remainingOperations;
			this.updateStatus();
		});

		database.addOnSettingsChangeHandlers(() => {
			this.updateStatus();
		});
	}

	private updateStatus(): void {
		this.statusBarItem.empty();
		const container = this.statusBarItem.createDiv({
			cls: ["sync-status"],
		});

		let hasShownMessage = false;

		if ((this.lastRemaining ?? 0) > 0) {
			hasShownMessage = true;
			container.createSpan({ text: `${this.lastRemaining} ⏳` });
		}

		if ((this.lastHistoryStats?.success ?? 0) > 0) {
			hasShownMessage = true;
			container.createSpan({
				text: `${this.lastHistoryStats?.success ?? 0} ✅`,
			});
		}

		if ((this.lastHistoryStats?.error ?? 0) > 0) {
			hasShownMessage = true;
			container.createSpan({
				text: `${this.lastHistoryStats?.error ?? 0} ❌`,
			});
		}

		if (!hasShownMessage) {
			if (this.database.getSettings().isSyncEnabled) {
				container.createSpan({ text: "VaultLink is idle" });
			} else {
				const button = container.createEl("button", {
					text: "VaultLink is disabled, click to configure",
					cls: "initialize-button",
				});
				button.onclick = (): void => {
					this.plugin.openSettings();
				};
			}
		}
	}
}
