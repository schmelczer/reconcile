import { Editor, MarkdownView, Plugin, WorkspaceLeaf } from "obsidian";

import * as lib from "../../backend/sync_lib/pkg/sync_lib.js";
import * as wasmBin from "../../backend/sync_lib/pkg/sync_lib_bg.wasm";
import { SyncSettingsTab } from "./views/settings-tab";
import { SyncView } from "./views/sync-view";

import { Logger } from "./logger";
import { SyncEventHandler } from "./events/sync-event-handler";
import { SyncService } from "./services/sync_service";
import { Database } from "./database/database";
import { applyRemoteChangesLocally } from "./sync-operations/apply-remote-changes-locally";
import { ObsidianFileOperations } from "./file-operations/obsidian-file-operations";
import { applyLocalChangesRemotely } from "./sync-operations/apply-local-changes-remotely";

export default class SyncPlugin extends Plugin {
	private remoteListenerIntervalId: number | null = null;
	private operations = new ObsidianFileOperations(this.app.vault);

	async onload() {
		Logger.getInstance().info('Starting plugin "Sample Plugin"');

		await lib.default(
			Promise.resolve(
				(wasmBin as any).default // eslint-disable-line @typescript-eslint/no-explicit-any
			)
		);

		this.addCommand({
			id: "sample-editor-command",
			name: "Sample editor command",
			editorCallback: (editor: Editor, _view: MarkdownView) => {
				console.log(editor.getSelection());
				editor.replaceSelection("Sample Editor Command");
			},
		});

		const database = new Database(
			await this.loadData(),
			this.saveData.bind(this)
		);

		const syncServer = new SyncService(database);

		this.addSettingTab(
			new SyncSettingsTab(this.app, this, database, syncServer)
		);

		const eventHandler = new SyncEventHandler(
			database,
			syncServer,
			this.operations
		);

		this.app.workspace.onLayoutReady(() => {
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

			applyLocalChangesRemotely(database, syncServer, this.operations);
		});

		this.registerRemoteEventListener(
			database,
			syncServer,
			database.getSettings().fetchChangesUpdateIntervalMs
		);
		database.addOnSettingsChangeHandlers((settings, oldSettings) => {
			this.registerRemoteEventListener(
				database,
				syncServer,
				settings.fetchChangesUpdateIntervalMs
			);

			if (!oldSettings.isSyncEnabled && settings.isSyncEnabled) {
				applyLocalChangesRemotely(
					database,
					syncServer,
					this.operations
				);
			}
		});

		this.registerView(SyncView.TYPE, (leaf) => new SyncView(leaf));

		const ribbonIconEl = this.addRibbonIcon(
			"dice",
			"Sample Plugin",
			(_: MouseEvent) => this.activateView()
		);
		ribbonIconEl.addClass("my-plugin-ribbon-class");
	}

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

	onunload(): void {
		if (this.remoteListenerIntervalId) {
			window.clearInterval(this.remoteListenerIntervalId);
		}
	}

	private registerRemoteEventListener(
		database: Database,
		syncServer: SyncService,
		intervalMs: number
	) {
		if (this.remoteListenerIntervalId) {
			window.clearInterval(this.remoteListenerIntervalId);
		}

		this.remoteListenerIntervalId = window.setInterval(
			() =>
				applyRemoteChangesLocally(
					database,
					syncServer,
					this.operations
				),
			intervalMs
		);
	}
}
