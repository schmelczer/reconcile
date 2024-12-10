import {
	App,
	Editor,
	MarkdownView,
	Modal,
	Notice,
	Plugin,
	PluginSettingTab,
	Setting,
} from "obsidian";

import SyncPlugin from "src/plugin.js";
import { Database } from "src/database/database";
import { SyncServer } from "src/services/sync_service";

export class SyncSettingsTab extends PluginSettingTab {
	constructor(
		app: App,
		plugin: SyncPlugin,
		private database: Database,
		private syncServer: SyncServer
	) {
		super(app, plugin);
	}

	display(): void {
		const { containerEl } = this;

		containerEl.empty();

		new Setting(containerEl)
			.setName("Remote URL")
			.setDesc("Your server's URL")
			.setTooltip(
				"This is the URL of the server you want to sync with, todo, links to docs"
			)
			.addText((text) =>
				text
					.setPlaceholder("https://example.com:8080/obsidian")
					.setValue(this.database.getSettings().remoteUri)
					.onChange((value) =>
						this.database.setSetting("remoteUri", value)
					)
			)
			.addButton((button) =>
				button.setButtonText("Test Connection").onClick(async () => {
					try {
						const result = await this.syncServer.ping();
						new Notice(
							`Successfully connected to server! (server version: ${result.serverVersion})`
						);
					} catch (e) {
						new Notice("Failed to connect to server: " + e);
					}
				})
			);

		new Setting(containerEl)
			.setName("Access token")
			.setDesc(
				"Set the access token for the server that you can get from the server"
			)
			.setTooltip("todo, links to dcocs")
			.addTextArea((text) =>
				text
					.setPlaceholder("ey...")
					.setValue(this.database.getSettings().token)
					.onChange((value) =>
						this.database.setSetting("token", value)
					)
			);

		new Setting(containerEl)
			.setName("Full scan interval (seconds)")
			.setDesc(
				"How often would you like to do a full scan of the local files"
			)
			.setTooltip("todo, links to docs")
			.addToggle((toggle) =>
				toggle
					.setValue(this.database.getSettings().fullScanEnabled)
					.onChange((value) =>
						this.database.setSetting("fullScanEnabled", value)
					)
			)
			.addSlider((text) =>
				text
					.setLimits(1, 3600, 1)
					.setDynamicTooltip()
					.setValue(
						this.database.getSettings().fullScanIntervalInSeconds
					)
					.onChange((value) =>
						this.database.setSetting(
							"fullScanIntervalInSeconds",
							value
						)
					)
			);
	}
}
