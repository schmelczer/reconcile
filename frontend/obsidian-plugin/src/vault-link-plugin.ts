import type { WorkspaceLeaf } from "obsidian";
import { Plugin } from "obsidian";
import "./styles.scss";
import "../manifest.json";

import { SyncSettingsTab } from "./views/settings-tab";
import { HistoryView } from "./views/history-view";
import { ObsidianFileEventHandler } from "./obisidan-event-handler";
import { ObsidianFileOperations } from "./obsidian-file-operations";
import { StatusBar } from "./views/status-bar";

import { LogsView } from "./views/logs-view";
import { StatusDescription } from "./views/status-description";
import {
	applyRemoteChangesLocally,
	Database,
	Logger,
	Syncer,
	SyncHistory,
	SyncService,
	initialize
} from "sync-client";

export default class VaultLinkPlugin extends Plugin {
	private readonly operations = new ObsidianFileOperations(this.app.vault);
	private readonly history = new SyncHistory();
	private settingsTab: SyncSettingsTab;
	private remoteListenerIntervalId: number | null = null;

	public async onload(): Promise<void> {
		Logger.getInstance().info("Starting plugin");

		await initialize();

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
			syncer
		});
		this.addSettingTab(this.settingsTab);

		new StatusBar(database, this, this.history, syncer);

		this.registerView(
			HistoryView.TYPE,
			(leaf) => new HistoryView(leaf, database, this.history)
		);
		this.registerView(
			LogsView.TYPE,
			(leaf) => new LogsView(this, database, leaf)
		);

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
				)
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

		database.addOnSettingsChangeHandlers((settings, oldSettings) => {
			this.registerRemoteEventListener(
				database,
				syncService,
				syncer,
				settings.fetchChangesUpdateIntervalMs
			);

			if (!oldSettings.isSyncEnabled && settings.isSyncEnabled) {
				syncer
					.scheduleSyncForOfflineChanges()
					.catch((_error: unknown) => {
						Logger.getInstance().error(
							"Failed to schedule sync for offline changes"
						);
					});
			}
		});

		Logger.getInstance().info("Plugin loaded");
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
					syncer
				}),
			intervalMs
		);
	}
}
