import type { App } from "obsidian";
import { Notice, PluginSettingTab, Setting } from "obsidian";

import type VaultLinkPlugin from "src/vault-link-plugin";
import type { StatusDescription } from "./status-description";
import { LogsView } from "./logs-view";
import { HistoryView } from "./history-view";
import type { SyncService, Syncer, Database } from "sync-client";
import { Logger, LogLevel } from "sync-client";

export class SyncSettingsTab extends PluginSettingTab {
	private editedVaultName: string;

	private readonly plugin: VaultLinkPlugin;
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
		syncer
	}: {
		app: App;
		plugin: VaultLinkPlugin;
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

		this.renderSettingsHeader(containerEl);
		this.renderConnectionSettings(containerEl);
		this.renderSyncSettings(containerEl);
		this.renderViewSettings(containerEl);
	}

	public hide(): void {
		super.hide();
		this.setStatusDescriptionSubscription();
	}

	private renderSettingsHeader(containerEl: HTMLElement): void {
		containerEl.createEl("h2", { text: "VaultLink" }).createSpan({
			text: this.plugin.manifest.version,
			cls: "version"
		});

		containerEl.createDiv(
			{
				cls: "description"
			},
			(descriptionContainer) => {
				this.setStatusDescriptionSubscription((): void => {
					this.statusDescription.renderStatusDescription(
						descriptionContainer
					);
				});
			}
		);

		containerEl.createDiv(
			{
				cls: "button-container"
			},
			(buttonContainer) => {
				buttonContainer.createEl(
					"button",
					{
						text: "Show history"
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
						text: "Show logs"
					},
					(button) =>
						(button.onclick = async (): Promise<void> => {
							this.plugin.closeSettings();
							await this.plugin.activateView(LogsView.TYPE);
						})
				);
			}
		);
	}

	private renderConnectionSettings(containerEl: HTMLElement): void {
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
	}

	private renderSyncSettings(containerEl: HTMLElement): void {
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
			.setName("Maximum file size to be uploaded (MB)")
			.setDesc(
				"Set the maximum file size that can be uploaded to the server. Files larger than this size will be ignored."
			)
			.addSlider((slider) =>
				slider
					.setLimits(0, 32, 1)
					.setDynamicTooltip()
					.setInstant(false)
					.setValue(this.database.getSettings().maxFileSizeMB)
					.onChange(async (value) =>
						this.database.setSetting("maxFileSizeMB", value)
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

	private renderViewSettings(containerEl: HTMLElement): void {
		containerEl.createEl("h3", { text: "View" });

		new Setting(containerEl)
			.setName("Show no-op sync operations in history")
			.setDesc(
				"Enabling this will make the history view more verbose while also providing more explanation for the scyning choices made."
			)
			.addToggle((toggle) =>
				toggle
					.setValue(this.database.getSettings().displayNoopSyncEvents)
					.onChange(async (value) =>
						this.database.setSetting("displayNoopSyncEvents", value)
					)
			);

		new Setting(containerEl)
			.setName("Minimum log level")
			.setDesc(
				"Set the log level for the plugin. Lower levels will show more logs."
			)
			.addDropdown((dropdown) =>
				dropdown
					.addOptions({
						[LogLevel.DEBUG]: LogLevel.DEBUG,
						[LogLevel.INFO]: LogLevel.INFO,
						[LogLevel.WARNING]: LogLevel.WARNING,
						[LogLevel.ERROR]: LogLevel.ERROR
					})
					.setValue(this.database.getSettings().minimumLogLevel)
					.onChange(async (value) =>
						this.database.setSetting(
							"minimumLogLevel",
							// eslint-disable-next-line @typescript-eslint/no-unsafe-type-assertion
							value as LogLevel
						)
					)
			);
	}

	private setStatusDescriptionSubscription(
		newSubscription?: () => void
	): void {
		if (this.statusDescriptionSubscription) {
			this.statusDescription.removeStatusChangeListener(
				this.statusDescriptionSubscription
			);
		}
		this.statusDescriptionSubscription = newSubscription;
		if (this.statusDescriptionSubscription) {
			this.statusDescriptionSubscription();
			this.statusDescription.addStatusChangeListener(
				this.statusDescriptionSubscription
			);
		}
	}
}
