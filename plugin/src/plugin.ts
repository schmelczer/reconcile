import {
	App,
	Editor,
	MarkdownView,
	Modal,
	Notice,
	Plugin,
	PluginSettingTab,
	Setting,
	WorkspaceLeaf,
} from "obsidian";

import * as plugin from "../../backend/sync_lib/pkg/sync_lib.js";
import * as wasmBin from "../../backend/sync_lib/pkg/sync_lib_bg.wasm";
import { getSystemErrorName } from "util";
import { SyncSettingsTab } from "./settings/settings-tab.js";
import { SyncView } from "./views/sync-view.js";
import {
	DEFAULT_SETTINGS,
	SettingsContainer,
	SyncSettings,
} from "./settings/settings.js";
import { Logger } from "./logger.js";
import { SyncEventHandler } from "./events/sync-event-handler.js";
import { Syncer } from "./syncer/syncer.js";

export default class SyncPlugin extends Plugin {
	async onload() {
		Logger.getInstance().info('Starting plugin "Sample Plugin"');

		await plugin.default(Promise.resolve((wasmBin as any).default));

		// This adds an editor command that can perform some operation on the current editor instance
		this.addCommand({
			id: "sample-editor-command",
			name: "Sample editor command",
			editorCallback: (editor: Editor, view: MarkdownView) => {
				console.log(editor.getSelection());
				editor.replaceSelection("Sample Editor Command");
			},
		});

		const settingsContainer = new SettingsContainer(
			this,
			await this.loadData()
		);
		this.addSettingTab(
			new SyncSettingsTab(this.app, this, settingsContainer)
		);

		const syncer = new Syncer(settingsContainer);
		const eventHandler = new SyncEventHandler(syncer);

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
		].forEach((event) => this.registerEvent(event));

		// When registering intervals, this function will automatically clear the interval when the plugin is disabled.
		this.registerInterval(
			window.setInterval(() => console.log("setInterval"), 5 * 60 * 1000)
		);
		this.registerView(SyncView.TYPE, (leaf) => new SyncView(leaf));

		const ribbonIconEl = this.addRibbonIcon(
			"dice",
			"Sample Plugin",
			(evt: MouseEvent) => {
				this.activateView();

				new Notice("This is a notice!");
			}
		);
		ribbonIconEl.addClass("my-plugin-ribbon-class");
	}

	onunload() {}

	async activateView() {
		const { workspace } = this.app;

		let leaf: WorkspaceLeaf | null = null;
		const leaves = workspace.getLeavesOfType(SyncView.TYPE);

		if (leaves.length > 0) {
			// A leaf with our view already exists, use that
			leaf = leaves[0];
		} else {
			// Our view could not be found in the workspace, create a new leaf
			// in the right sidebar for it
			leaf = workspace.getRightLeaf(false);
			await leaf?.setViewState({ type: SyncView.TYPE, active: true });
		}

		// "Reveal" the leaf in case it is in a collapsed sidebar
		workspace.revealLeaf(leaf!);
	}
}
