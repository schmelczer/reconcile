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
import { Logger, SyncClient } from "sync-client";

export default class VaultLinkPlugin extends Plugin {
	private settingsTab: SyncSettingsTab | undefined;
	private client!: SyncClient;

	public async onload(): Promise<void> {
		Logger.getInstance().info("Starting plugin");

		this.client = await SyncClient.create(
			new ObsidianFileOperations(this.app.vault),
			{
				load: this.loadData.bind(this),
				save: this.saveData.bind(this)
			}
		);

		const statusDescription = new StatusDescription(this.client);

		this.settingsTab = new SyncSettingsTab({
			app: this.app,
			plugin: this,
			syncClient: this.client,
			statusDescription
		});
		this.addSettingTab(this.settingsTab);

		new StatusBar(this, this.client);

		this.registerView(
			HistoryView.TYPE,
			(leaf) =>
				new HistoryView(leaf, this.client.settings, this.client.history)
		);
		this.registerView(
			LogsView.TYPE,
			(leaf) => new LogsView(this, this.client.settings, leaf)
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

		const eventHandler = new ObsidianFileEventHandler(this.client.syncer);

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

			void this.client.syncer.scheduleSyncForOfflineChanges();
		});
	}

	public onunload(): void {
		this.client.onunload();
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
}
