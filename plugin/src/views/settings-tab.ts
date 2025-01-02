import type { App } from "obsidian";
import { Notice, PluginSettingTab, Setting } from "obsidian";

import type SyncPlugin from "src/plugin";
import type { Database } from "src/database/database";
import type { SyncService } from "src/services/sync-service";
import { Logger } from "src/tracing/logger";
import type { Syncer } from "src/sync-operations/syncer";
import type { StatusDescription } from "./status-description";
import { LogsView } from "./logs-view";
import { HistoryView } from "./history-view";

export class SyncSettingsTab extends PluginSettingTab {
	private editedVaultName: string;

	private readonly plugin: SyncPlugin;
	private readonly database: Database;
	private readonly syncService: SyncService;
	private readonly statusDescription: StatusDescription;
	private readonly syncer: Syncer;
	private statusDescriptionSubscription: (() => void) | undefined;

	public constructor({
		app,
		plugin,
		database,
		syncService,
		statusDescription,
		syncer,
	}: {
		app: App;
		plugin: SyncPlugin;
		database: Database;
		syncService: SyncService;
		statusDescription: StatusDescription;
		syncer: Syncer;
	}) {
		super(app, plugin);
		this.plugin = plugin;
		this.database = database;
		this.syncService = syncService;
		this.statusDescription = statusDescription;
		this.syncer = syncer;

		this.editedVaultName = this.database.getSettings().vaultName;
		this.database.addOnSettingsChangeHandlers(
			(newSettings, oldSettings) => {
				if (newSettings.vaultName !== oldSettings.vaultName) {
					this.editedVaultName = newSettings.vaultName;
					this.display();
				}
			}
		);
	}

	public display(): void {
		const { containerEl } = this;
		containerEl.empty();
		containerEl.addClass("vault-link-settings");

		containerEl.createEl("h2", { text: "VaultLink" }).createSpan({
			text: this.plugin.manifest.version,
			cls: "version",
		});

		const descriptionContainer = containerEl.createDiv({
			cls: "description",
		});
		this.statusDescriptionSubscription = (): void => {
			this.statusDescription.renderStatusDescription(
				descriptionContainer
			);
		};
		this.statusDescription.addStatusChangeListener(
			this.statusDescriptionSubscription
		);

		containerEl.createDiv(
			{
				cls: "button-container",
			},
			(buttonContainer) => {
				buttonContainer.createEl(
					"button",
					{
						text: "Show history",
					},
					(button) =>
						(button.onclick = async (): Promise<void> => {
							this.plugin.closeSettings();
							await this.plugin.activateView(HistoryView.TYPE);
						})
				);

				buttonContainer.createEl(
					"button",
					{
						text: "Show logs",
					},
					(button) =>
						(button.onclick = async (): Promise<void> => {
							this.plugin.closeSettings();
							await this.plugin.activateView(LogsView.TYPE);
						})
				);
			}
		);

		containerEl.createEl("h3", { text: "Connection" });

		new Setting(containerEl)
			.setName("Server address")
			.setDesc(
				"Your VaultLink server's URL including the protocol and full path."
			)
			.setTooltip("This is the URL of the server you want to sync with.")
			.addText((text) =>
				text
					.setPlaceholder("https://example.com:3030")
					.setValue(this.database.getSettings().remoteUri)
					.onChange(async (value) =>
						this.database.setSetting("remoteUri", value)
					)
			)
			.addButton((button) =>
				button.setButtonText("Test connection").onClick(async () => {
					new Notice(
						(await this.syncService.checkConnection()).message
					);
					await this.statusDescription.updateConnectionState();
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
					.onChange(async (value) =>
						this.database.setSetting("token", value)
					)
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
					await this.database.setSetting(
						"vaultName",
						this.editedVaultName
					);
					await this.syncer.reset();
					Logger.getInstance().reset();
					new Notice(
						"Sync state has been reset, you will need to resync"
					);
				})
			);

		containerEl.createEl("h3", { text: "Sync" });

		new Setting(containerEl)
			.setName("Danger zone")
			.setDesc(
				"How many concurrent sync operations to run. Setting this value higher may increase the overall performance, however, it will require more memory as well. If you notice frequent crashes, especially on mobile, set this to 1."
			)
			.addButton((button) =>
				button.setButtonText("Reset sync state").onClick(async () => {
					await this.syncer.reset();
					Logger.getInstance().reset();
					new Notice(
						"Sync state has been reset, you will need to resync"
					);
				})
			);

		new Setting(containerEl)
			.setName("Remote fetching frequency (seconds)")
			.setDesc(
				"Set how often should the plugin check for changes on the server. Lower values will increase the frequency of the checks making it easier to collaborate with others."
			)
			.setTooltip("todo, links to docs")
			.addSlider((text) =>
				text
					.setLimits(0.5, 60, 0.5)
					.setDynamicTooltip()
					.setInstant(false)
					.setValue(
						this.database.getSettings()
							.fetchChangesUpdateIntervalMs / 1000
					)
					.onChange(async (value) =>
						this.database.setSetting(
							"fetchChangesUpdateIntervalMs",
							value * 1000
						)
					)
			);

		new Setting(containerEl)
			.setName("Sync concurrency")
			.setDesc(
				"How many concurrent sync operations to run. Setting this value higher may increase the overall performance, however, it will require more memory as well. If you notice frequent crashes, especially on mobile, set this to 1."
			)
			.addSlider((text) =>
				text
					.setLimits(1, 16, 1)
					.setDynamicTooltip()
					.setInstant(false)
					.setValue(this.database.getSettings().syncConcurrency)
					.onChange(async (value) =>
						this.database.setSetting("syncConcurrency", value)
					)
			);

		new Setting(containerEl)
			.setName("Enable sync")
			.setDesc(
				"Enable pulling and pushing changes to the remote server. The first time it's enabled, or after the sync state has been reset, all local files will be pushed to the server."
			)
			.setTooltip(
				"Enable pulling and pushing changes to the remote server."
			)
			.addToggle((toggle) =>
				toggle
					.setValue(this.database.getSettings().isSyncEnabled)
					.onChange(async (value) =>
						this.database.setSetting("isSyncEnabled", value)
					)
			);
	}

	public hide(): void {
		super.hide();

		if (this.statusDescriptionSubscription) {
			this.statusDescription.removeStatusChangeListener(
				this.statusDescriptionSubscription
			);
		}
	}
}
