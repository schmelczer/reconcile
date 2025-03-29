import "./status-bar.scss";

import type { HistoryStats, SyncClient } from "sync-client";
import type VaultLinkPlugin from "../../vault-link-plugin";

export class StatusBar {
	private readonly statusBarItem: HTMLElement;

	private lastHistoryStats: HistoryStats | undefined;
	private lastRemaining: number | undefined;

	public constructor(
		private readonly plugin: VaultLinkPlugin,
		private readonly syncClient: SyncClient
	) {
		this.statusBarItem = plugin.addStatusBarItem();
		this.syncClient.addSyncHistoryUpdateListener((status) => {
			this.lastHistoryStats = status;
			this.updateStatus();
		});

		this.syncClient.addRemainingSyncOperationsListener(
			(remainingOperations) => {
				this.lastRemaining = remainingOperations;
				this.updateStatus();
			}
		);

		this.syncClient.addOnSettingsChangeListener(() => {
			this.updateStatus();
		});
	}

	private updateStatus(): void {
		this.statusBarItem.empty();
		const container = this.statusBarItem.createDiv({
			cls: ["sync-status"]
		});

		if (!this.syncClient.getSettings().isSyncEnabled) {
			const button = container.createEl("button", {
				text: "VaultLink is disabled, click to configure",
				cls: "initialize-button"
			});
			button.onclick = (): void => {
				this.plugin.openSettings();
			};

			return;
		}

		let hasShownMessage = false;

		if ((this.lastRemaining ?? 0) > 0) {
			hasShownMessage = true;
			container.createSpan({ text: `${this.lastRemaining} ⏳` });
		}

		if ((this.lastHistoryStats?.success ?? 0) > 0) {
			hasShownMessage = true;
			container.createSpan({
				text: `${this.lastHistoryStats?.success ?? 0} ✅`
			});
		}

		if ((this.lastHistoryStats?.error ?? 0) > 0) {
			hasShownMessage = true;
			container.createSpan({
				text: `${this.lastHistoryStats?.error ?? 0} ❌`
			});
		}

		if (!hasShownMessage) {
			container.createSpan({ text: "VaultLink is idle" });
		}
	}
}
