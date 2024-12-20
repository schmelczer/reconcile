import type { WorkspaceLeaf } from "obsidian";
import { Plugin } from "obsidian";

import * as lib from "../../backend/sync_lib/pkg/sync_lib.js";
import * as wasmBin from "../../backend/sync_lib/pkg/sync_lib_bg.wasm";
import { SyncSettingsTab } from "./views/settings-tab";
import { HistoryView } from "./views/history-view.js";

import { ObsidianFileEventHandler } from "./events/obisidan-event-handler.js";
import { SyncService } from "./services/sync-service";
import { Database } from "./database/database";
import { applyRemoteChangesLocally } from "./sync-operations/apply-remote-changes-locally";
import { ObsidianFileOperations } from "./file-operations/obsidian-file-operations";
import { applyLocalChangesRemotely } from "./sync-operations/apply-local-changes-remotely";
import { StatusBar } from "./views/status-bar";
import { Logger } from "./tracing/logger.js";
import { SyncHistory } from "./tracing/sync-history.js";
import { LogsView } from "./views/logs-view.js";

export default class SyncPlugin extends Plugin {
	private remoteListenerIntervalId: number | null = null;
	private readonly operations = new ObsidianFileOperations(this.app.vault);
	private readonly history = new SyncHistory();

	public async onload(): Promise<void> {
		Logger.getInstance().info("Starting plugin");

		await lib.default(
			Promise.resolve(
				// eslint-disable-next-line
				(wasmBin as any).default
			)
		);

		const database = new Database(
			await this.loadData(),
			this.saveData.bind(this)
		);

		const syncServer = new SyncService(database);

		new StatusBar(this, this.history);

		this.addSettingTab(
			new SyncSettingsTab(
				this.app,
				this,
				database,
				syncServer,
				this.history
			)
		);

		const eventHandler = new ObsidianFileEventHandler(
			database,
			syncServer,
			this.operations,
			this.history
		);

		this.app.workspace.onLayoutReady(async () => {
			Logger.getInstance().info("Initialising sync handlers");

			[
				this.app.vault.on(
					"create",
					eventHandler.onCreate.bind(eventHandler)
				),
				this.app.vault.on(
					"modify",
					eventHandler.onModify.bind(eventHandler)
				),
				this.app.vault.on(
					"delete",
					eventHandler.onDelete.bind(eventHandler)
				),
				this.app.vault.on(
					"rename",
					eventHandler.onRename.bind(eventHandler)
				),
			].forEach((event) => {
				this.registerEvent(event);
			});

			await applyLocalChangesRemotely({
				database,
				syncServer,
				operations: this.operations,
				history: this.history,
			});

			Logger.getInstance().info("Sync handlers initialised");
		});

		this.registerRemoteEventListener(
			database,
			syncServer,
			database.getSettings().fetchChangesUpdateIntervalMs
		);

		// eslint-disable-next-line @typescript-eslint/no-misused-promises
		database.addOnSettingsChangeHandlers(async (settings, oldSettings) => {
			this.registerRemoteEventListener(
				database,
				syncServer,
				settings.fetchChangesUpdateIntervalMs
			);

			if (!oldSettings.isSyncEnabled && settings.isSyncEnabled) {
				await applyLocalChangesRemotely({
					database: database,
					syncServer,
					operations: this.operations,
					history: this.history,
				});
			}
		});

		this.registerView(
			HistoryView.TYPE,
			(leaf) => new HistoryView(leaf, this.history)
		);
		this.registerView(LogsView.TYPE, (leaf) => new LogsView(leaf));

		this.addRibbonIcon(
			HistoryView.ICON,
			"Open VaultLink events",
			async (_: MouseEvent) => this.activateView(HistoryView.TYPE)
		);
		this.addRibbonIcon(
			LogsView.ICON,
			"Open VaultLink logs",
			async (_: MouseEvent) => this.activateView(LogsView.TYPE)
		);

		Logger.getInstance().info("Plugin loaded");
	}

	public onunload(): void {
		if (this.remoteListenerIntervalId !== null) {
			window.clearInterval(this.remoteListenerIntervalId);
		}
	}

	private async activateView(type: string): Promise<void> {
		const { workspace } = this.app;

		let leaf: WorkspaceLeaf | null = null;
		const leaves = workspace.getLeavesOfType(type);

		if (leaves.length > 0) {
			[leaf] = leaves;
		} else {
			leaf = workspace.getRightLeaf(false);
			await leaf?.setViewState({ type: type, active: true });
		}

		if (leaf) {
			await workspace.revealLeaf(leaf);
		}
	}

	private registerRemoteEventListener(
		database: Database,
		syncServer: SyncService,
		intervalMs: number
	): void {
		if (this.remoteListenerIntervalId !== null) {
			window.clearInterval(this.remoteListenerIntervalId);
		}

		this.remoteListenerIntervalId = window.setInterval(
			// eslint-disable-next-line @typescript-eslint/no-misused-promises
			async () =>
				applyRemoteChangesLocally({
					database,
					syncServer,
					operations: this.operations,
					history: this.history,
				}),
			intervalMs
		);
	}
}
