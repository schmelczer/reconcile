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
import { StatusBar } from "./views/status-bar";
import { Logger } from "./tracing/logger.js";
import { SyncHistory } from "./tracing/sync-history.js";
import { LogsView } from "./views/logs-view.js";
import { Syncer } from "./sync-operations/syncer.js";
import { StatusDescription } from "./views/status-description.js";

export default class SyncPlugin extends Plugin {
	private readonly operations = new ObsidianFileOperations(this.app.vault);
	private readonly history = new SyncHistory();
	private settingsTab: SyncSettingsTab;
	private remoteListenerIntervalId: number | null = null;

	public async onload(): Promise<void> {
		Logger.getInstance().info("Starting plugin");

		await lib.default(
			Promise.resolve(
				// eslint-disable-next-line
				(wasmBin as any).default
			)
		);

		lib.setPanicHook();

		const database = new Database(
			await this.loadData(),
			this.saveData.bind(this)
		);

		const syncService = new SyncService(database);

		const syncer = new Syncer(
			database,
			syncService,
			this.operations,
			this.history
		);

		const statusDescription = new StatusDescription(
			database,
			syncService,
			this.history,
			syncer
		);

		this.settingsTab = new SyncSettingsTab({
			app: this.app,
			plugin: this,
			database,
			syncService,
			statusDescription,
			syncer,
		});
		this.addSettingTab(this.settingsTab);

		new StatusBar(database, this, this.history, syncer);

		const eventHandler = new ObsidianFileEventHandler(syncer);

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

			Logger.getInstance().info("Sync handlers initialised");

			void syncer.scheduleSyncForOfflineChanges();
		});

		this.registerRemoteEventListener(
			database,
			syncService,
			syncer,
			database.getSettings().fetchChangesUpdateIntervalMs
		);

		// eslint-disable-next-line @typescript-eslint/no-misused-promises
		database.addOnSettingsChangeHandlers(async (settings, oldSettings) => {
			this.registerRemoteEventListener(
				database,
				syncService,
				syncer,
				settings.fetchChangesUpdateIntervalMs
			);

			if (!oldSettings.isSyncEnabled && settings.isSyncEnabled) {
				await syncer.scheduleSyncForOfflineChanges();
			}
		});

		this.registerView(
			HistoryView.TYPE,
			(leaf) => new HistoryView(leaf, database, this.history)
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

		this.openSettings();
	}

	public onunload(): void {
		if (this.remoteListenerIntervalId !== null) {
			window.clearInterval(this.remoteListenerIntervalId);
		}
	}

	public openSettings(): void {
		// eslint-disable-next-line
		(this.app as any).setting.open(); // this is undocumented
		// eslint-disable-next-line
		(this.app as any).setting.openTab(this.settingsTab); // this is undocumented
	}

	public closeSettings(): void {
		// eslint-disable-next-line
		(this.app as any).setting.close(); // this is undocumented
	}

	public async activateView(type: string): Promise<void> {
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
		syncService: SyncService,
		syncer: Syncer,
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
					syncService,
					syncer,
				}),
			intervalMs
		);
	}
}
