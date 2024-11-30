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
import { SettingsContainer } from "./settings";

export class SyncSettingsTab extends PluginSettingTab {
	plugin: SyncPlugin;

	constructor(
		app: App,
		plugin: SyncPlugin,
		private settingsContainer: SettingsContainer
	) {
		super(app, plugin);
		this.plugin = plugin;
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
					.setValue(this.settingsContainer.getSettings().remoteUri)
					.onChange((value) =>
						this.settingsContainer.setSetting("remoteUri", value)
					)
			)
			.addButton((button) => button.setButtonText("Test Connection"));

		new Setting(containerEl)
			.setName("Access token")
			.setDesc(
				"Set the access token for the server that you can get from the server"
			)
			.setTooltip("todo, links to docs")
			.addTextArea((text) =>
				text
					.setPlaceholder("ey...")
					.setValue(this.settingsContainer.getSettings().token)
					.onChange((value) =>
						this.settingsContainer.setSetting("token", value)
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
					.setValue(
						this.settingsContainer.getSettings().fullScanEnabled
					)
					.onChange((value) =>
						this.settingsContainer.setSetting(
							"fullScanEnabled",
							value
						)
					)
			)
			.addSlider((text) =>
				text
					.setLimits(1, 3600, 1)
					.setDynamicTooltip()
					.setValue(
						this.settingsContainer.getSettings()
							.fullScanIntervalInSeconds
					)
					.onChange((value) =>
						this.settingsContainer.setSetting(
							"fullScanIntervalInSeconds",
							value
						)
					)
			);
	}
}
