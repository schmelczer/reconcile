import type {
	Editor,
	MarkdownFileInfo,
	MarkdownView,
	TAbstractFile,
	WorkspaceLeaf
} from "obsidian";
import { Platform, Plugin, TFile } from "obsidian";
import "../manifest.json";
import { HistoryView } from "./views/history/history-view";
import { StatusBar } from "./views/status-bar/status-bar";
import { LogsView } from "./views/logs/logs-view";
import { StatusDescription } from "./views/status-description/status-description";
import { SyncClient, rateLimit } from "sync-client";
import { ObsidianFileSystemOperations } from "./obsidian-file-system";
import { SyncSettingsTab } from "./views/settings/settings-tab";
import { registerConsoleForLogging } from "./utils/register-console-for-logging";
import { updateEditorStatusDisplay } from "./views/editor-sync-line/editor-sync-line";

export default class VaultLinkPlugin extends Plugin {
	private readonly disposables: (() => void)[] = [];
	private settingsTab: SyncSettingsTab | undefined;
	private client!: SyncClient;
	private readonly rateLimitedUpdatesPerFile = new Map<
		string,
		() => Promise<unknown>
	>();

	public async onload(): Promise<void> {
		this.client = await SyncClient.create({
			fs: new ObsidianFileSystemOperations(
				this.app.vault,
				this.app.workspace
			),
			persistence: {
				load: this.loadData.bind(this),
				save: this.saveData.bind(this)
			},
			nativeLineEndings: Platform.isWin ? "\r\n" : "\n"
		});

		registerConsoleForLogging(this.client);

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
			(leaf) => new LogsView(this.client, leaf)
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

		this.app.workspace.onLayoutReady(async () => {
			this.registerEditorEvents();
			void this.client.start();

			const interval = setInterval(() => {
				updateEditorStatusDisplay(this.app.workspace, this.client);
			}, 200);
			this.disposables.push(() => {
				clearInterval(interval);
			});
		});
	}

	public onunload(): void {
		this.client.stop();
		this.disposables.forEach((disposable) => {
			disposable();
		});
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

	private registerEditorEvents(): void {
		[
			this.app.workspace.on(
				"editor-change",
				async (
					_editor: Editor,
					info: MarkdownView | MarkdownFileInfo
				) => {
					const { file } = info;
					if (file) {
						await this.rateLimitedUpdate(file.path);
					}
				}
			),
			this.app.vault.on("create", async (file: TAbstractFile) => {
				if (file instanceof TFile) {
					await this.client.syncLocallyCreatedFile(file.path);
				}
			}),
			this.app.vault.on("modify", async (file: TAbstractFile) => {
				if (file instanceof TFile) {
					await this.rateLimitedUpdate(file.path);
				}
			}),
			this.app.vault.on("delete", async (file: TAbstractFile) => {
				await this.client.syncLocallyDeletedFile(file.path);
			}),
			this.app.vault.on(
				"rename",
				async (file: TAbstractFile, oldPath: string) => {
					if (file instanceof TFile) {
						await this.client.syncLocallyUpdatedFile({
							oldPath,
							relativePath: file.path
						});
					}
				}
			)
		].forEach((event) => {
			this.registerEvent(event);
		});
	}

	private async rateLimitedUpdate(path: string): Promise<void> {
		if (!this.rateLimitedUpdatesPerFile.has(path)) {
			this.rateLimitedUpdatesPerFile.set(
				path,
				rateLimit(
					async () =>
						this.client.syncLocallyUpdatedFile({
							relativePath: path
						}),
					250
				)
			);
		}
		await this.rateLimitedUpdatesPerFile.get(path)?.();
	}
}
