import type { Database } from "src/database/database";
import type {
	CheckConnectionResult,
	SyncService
} from "src/services/sync-service";
import type { Syncer } from "src/sync-operations/syncer";
import type { HistoryStats, SyncHistory } from "src/tracing/sync-history";

export class StatusDescription {
	private lastHistoryStats: HistoryStats | undefined;
	private lastRemaining: number | undefined;
	private lastConnectionState: CheckConnectionResult | undefined;

	private statusChangeListeners: (() => void)[] = [];

	public constructor(
		private readonly database: Database,
		private readonly syncService: SyncService,
		history: SyncHistory,
		syncer: Syncer
	) {
		void this.updateConnectionState();

		history.addSyncHistoryUpdateListener((status) => {
			this.lastHistoryStats = status;
			this.updateDescription();
		});

		syncer.addRemainingOperationsListener((remainingOperations) => {
			this.lastRemaining = remainingOperations;
			this.updateDescription();
		});

		database.addOnSettingsChangeHandlers(() => {
			void this.updateConnectionState();
		});
	}

	public async updateConnectionState(): Promise<void> {
		this.lastConnectionState = await this.syncService.checkConnection();
		this.updateDescription();
	}

	public addStatusChangeListener(listener: () => void): void {
		this.statusChangeListeners.push(listener);
	}
	public removeStatusChangeListener(listener: () => void): void {
		this.statusChangeListeners = this.statusChangeListeners.filter(
			(l) => l !== listener
		);
	}

	public renderStatusDescription(container: HTMLElement): void {
		container.empty();
		container.addClass("status-description");

		if (this.lastConnectionState == undefined) {
			container.createSpan({
				text: "VaultLink is starting up…",
				cls: "warning"
			});
			return;
		}

		if (!this.lastConnectionState.isSuccessful) {
			container.createSpan({
				text: `VaultLink failed to connect to the remote server with the error "${this.lastConnectionState.message}"`,
				cls: "error"
			});
			return;
		}

		container.createSpan({ text: "VaultLink is connected to the server " });
		container.createEl("a", {
			text: this.database.getSettings().remoteUri,
			href: this.database.getSettings().remoteUri
		});

		container.createSpan({
			text: ` and has indexed approximately `
		});
		container.createSpan({
			text: `${this.database.getDocuments().size}`,
			cls: "number"
		});
		container.createSpan({
			text: ` documents. `
		});

		if (
			(this.lastRemaining ?? 0) === 0 &&
			(this.lastHistoryStats?.success ?? 0) === 0 &&
			(this.lastHistoryStats?.error ?? 0) === 0
		) {
			if (this.database.getSettings().isSyncEnabled) {
				container.createSpan({
					text: "Syncing is enabled but VaultLink hasn't found anything to sync yet."
				});
			} else {
				container.createSpan({
					text: "However, syncing is disabled right now.",
					cls: "warning"
				});
			}
			return;
		}

		container.createSpan({
			text: "The plugin has "
		});
		container.createSpan({
			text: `${this.lastRemaining ?? 0}`,
			cls: "number"
		});
		container.createSpan({
			text: " outstanding operations while having succeeded "
		});
		container.createSpan({
			text: `${this.lastHistoryStats?.success ?? 0}`,
			cls: ["number", "good"]
		});
		container.createSpan({
			text: " times and failed "
		});
		container.createSpan({
			text: `${this.lastHistoryStats?.error ?? 0}`,
			cls: ["number", "bad"]
		});
		container.createSpan({
			text: " times."
		});
	}

	private updateDescription(): void {
		this.statusChangeListeners.forEach((listener) => {
			listener();
		});
	}
}
