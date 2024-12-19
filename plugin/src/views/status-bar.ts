import { Plugin } from "obsidian";
import { RequestCountStatus, SyncService } from "src/services/sync-service";

export class StatusBar {
	private statusBarItem: HTMLElement;

	public constructor(plugin: Plugin, service: SyncService) {
		this.statusBarItem = plugin.addStatusBarItem();
		service.addRequestCountChangeListener((status) =>
			this.updateStatus(status)
		);
	}

	private updateStatus({
		waiting,
		success,
		failure,
	}: RequestCountStatus): void {
		this.statusBarItem.setText(`${waiting} 🔄 ${success} ✅ ${failure} ❌`);
	}
}
