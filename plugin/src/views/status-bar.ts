import type { Plugin } from "obsidian";
import type { Syncer } from "src/sync-operations/syncer";
import type { HistoryStats, SyncHistory } from "src/tracing/sync-history";

export class StatusBar {
	private readonly statusBarItem: HTMLElement;

	private lastHistoryStats: HistoryStats | undefined;
	private lastRemaining: number | undefined;

	public constructor(plugin: Plugin, history: SyncHistory, syncer: Syncer) {
		this.statusBarItem = plugin.addStatusBarItem();
		history.addSyncHistoryUpdateListener((status) => {
			this.lastHistoryStats = status;
			this.updateStatus();
		});

		syncer.addRemainingOperationsListener((remainingOperations) => {
			this.lastRemaining = remainingOperations;
			this.updateStatus();
		});
	}

	private updateStatus(): void {
		this.statusBarItem.setText(
			`${this.lastRemaining ?? 0} ⏳ | ${
				this.lastHistoryStats?.success ?? 0
			} ✅ | ${this.lastHistoryStats?.error ?? 0} ❌`
		);
	}
}
