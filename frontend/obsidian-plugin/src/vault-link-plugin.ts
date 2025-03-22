import type { WorkspaceLeaf } from "obsidian";
import { Platform, Plugin } from "obsidian";
import "./styles.scss";
import "../manifest.json";
import { SyncSettingsTab } from "./views/settings-tab";
import { HistoryView } from "./views/history-view";
import { ObsidianFileEventHandler } from "./obisidan-event-handler";
import { StatusBar } from "./views/status-bar";
import { LogsView } from "./views/logs-view";
import { StatusDescription } from "./views/status-description";
import type { LogLine } from "sync-client";
import { SyncClient, LogLevel } from "sync-client";
import { ObsidianFileSystemOperations } from "./obsidian-file-system";

export default class VaultLinkPlugin extends Plugin {
	private settingsTab: SyncSettingsTab | undefined;
	private client!: SyncClient;
	private static registerConsoleForLogging(client: SyncClient): void {
		client.logger.addOnMessageListener((logLine: LogLine) => {
			const formatted = `${logLine.timestamp.toISOString()} ${logLine.level} ${logLine.message}`;

			switch (logLine.level) {
				case LogLevel.ERROR:
					console.error(formatted);
					break;
				case LogLevel.WARNING:
					console.warn(formatted);
					break;
				case LogLevel.INFO:
					console.info(formatted);
					break;
				case LogLevel.DEBUG:
					console.debug(formatted);
					break;
			}
		});
	}

	public async onload(): Promise<void> {
		this.client = await SyncClient.create({
			fs: new ObsidianFileSystemOperations(this.app.vault),
			persistence: {
				load: this.loadData.bind(this),
				save: this.saveData.bind(this)
			},
			nativeLineEndings: Platform.isWin ? "\r\n" : "\n"
		});

		VaultLinkPlugin.registerConsoleForLogging(this.client);

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
			(leaf) => new HistoryView(leaf, this.client)
		);
		this.registerView(
			LogsView.TYPE,
			(leaf) => new LogsView(this, this.client, leaf)
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

		const eventHandler = new ObsidianFileEventHandler(this.client);

		this.app.workspace.onLayoutReady(async () => {
			this.client.logger.info("Initialising sync handlers");

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

			void this.client.start();

			this.client.logger.info("Sync handlers initialised");
		});
	}

	public onunload(): void {
		this.client.stop();
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
