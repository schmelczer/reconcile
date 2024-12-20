import type { Plugin } from "obsidian";
import type { HistoryStats, SyncHistory } from "src/tracing/sync-history";

export class StatusBar {
	private readonly statusBarItem: HTMLElement;

	public constructor(plugin: Plugin, history: SyncHistory) {
		this.statusBarItem = plugin.addStatusBarItem();
		history.addSyncHistoryUpdateListener((status) => {
			this.updateStatus(status);
		});
	}

	private updateStatus({ success, error }: HistoryStats): void {
		this.statusBarItem.setText(`${success} ✅ ${error} ❌`);
	}
}
