import { App, Notice, PluginSettingTab, Setting } from "obsidian";

import SyncPlugin from "src/plugin";
import { Database } from "src/database/database";
import { SyncService } from "src/services/sync-service";

export class SyncSettingsTab extends PluginSettingTab {
	private editedVaultName: string;

	public constructor(
		app: App,
		plugin: SyncPlugin,
		private database: Database,
		private syncServer: SyncService
	) {
		super(app, plugin);
		this.editedVaultName = this.database.getSettings().vaultName;
		this.database.addOnSettingsChangeHandlers((s) => {
			this.editedVaultName = s.vaultName;
			this.display();
		});
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
						if (result.isAuthenticated) {
							new Notice(
								`Successfully authenticated with the server (version: ${result.serverVersion})!`
							);
						} else {
							new Notice(
								`Successfully connected to server (version: ${result.serverVersion}) but failed to authenticate.`
							);
						}
					} catch (e) {
						new Notice("Failed to connect to server: " + e);
					}
				})
			)
			.addSlider((text) =>
				text
					.setLimits(1, 3600, 1)
					.setValue(5)
					.setDynamicTooltip()
					.setInstant(false)
					.setValue(this.database.getSettings().uploadConcurrency)
					.onChange((value) =>
						this.database.setSetting("uploadConcurrency", value)
					)
			)
			.addButton((button) =>
				button.setButtonText("Reset sync state").onClick(async () => {
					await this.database.resetSyncState();
					new Notice(
						"Sync state has been reset, you will need to resync"
					);
				})
			);

		new Setting(containerEl)
			.setName("Vault name")
			.setDesc(
				"Set the name of the remote vault that you want to sync with"
			)
			.setTooltip("todo, links to dcocs")
			.addText((text) =>
				text
					.setPlaceholder("My Obsidian Vault")
					.setValue(this.database.getSettings().vaultName)
					.onChange((value) => (this.editedVaultName = value))
			)
			.addButton((button) =>
				button.setButtonText("Apply").onClick(async () => {
					if (
						this.editedVaultName ===
						this.database.getSettings().vaultName
					) {
						return;
					}
					this.database.setSetting("vaultName", this.editedVaultName);
					await this.database.resetSyncState();
					new Notice(
						"Sync state has been reset, you will need to resync"
					);
				})
			);

		new Setting(containerEl)
			.setName("Access token")
			.setClass("sync-settings-access-token")
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
					.setValue(this.database.getSettings().isSyncEnabled)
					.onChange((value) =>
						this.database.setSetting("isSyncEnabled", value)
					)
			)
			.addSlider((text) =>
				text
					.setLimits(1, 3600, 1)
					.setDynamicTooltip()
					.setInstant(false)
					.setValue(
						this.database.getSettings().fetchChangesUpdateIntervalMs
					)
					.onChange((value) =>
						this.database.setSetting(
							"fetchChangesUpdateIntervalMs",
							value
						)
					)
			);
	}
}
