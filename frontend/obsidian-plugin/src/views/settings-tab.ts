import type { App } from "obsidian";
import { Notice, PluginSettingTab, Setting } from "obsidian";
import type VaultLinkPlugin from "../vault-link-plugin";
import type { StatusDescription } from "./status-description";
import { LogsView } from "./logs-view";
import { HistoryView } from "./history-view";
import type { SyncClient, SyncSettings } from "sync-client";

export class SyncSettingsTab extends PluginSettingTab {
	private editedServerUri: string;
	private editedToken: string;
	private editedVaultName: string;

	private readonly plugin: VaultLinkPlugin;
	private readonly syncClient: SyncClient;
	private readonly statusDescription: StatusDescription;
	private statusDescriptionSubscription: (() => void) | undefined;

	public constructor({
		app,
		plugin,
		syncClient,
		statusDescription
	}: {
		app: App;
		plugin: VaultLinkPlugin;
		syncClient: SyncClient;
		statusDescription: StatusDescription;
	}) {
		super(app, plugin);
		this.plugin = plugin;
		this.syncClient = syncClient;
		this.statusDescription = statusDescription;

		this.editedServerUri = this.syncClient.getSettings().remoteUri;
		this.editedToken = this.syncClient.getSettings().token;
		this.editedVaultName = this.syncClient.getSettings().vaultName;

		this.syncClient.addOnSettingsChangeListener(
			(newSettings, oldSettings) => {
				let hasChanged = false;

				if (newSettings.remoteUri !== oldSettings.remoteUri) {
					this.editedServerUri = newSettings.remoteUri;
					hasChanged = true;
				}

				if (newSettings.token !== oldSettings.token) {
					this.editedToken = newSettings.token;
					hasChanged = true;
				}

				if (newSettings.vaultName !== oldSettings.vaultName) {
					this.editedVaultName = newSettings.vaultName;
					hasChanged = true;
				}

				if (hasChanged) {
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

		const [title, updateTitle] = this.unsavedAwareSettingName(
			"Server address",
			"remoteUri"
		);

		new Setting(containerEl)
			.setName(title)
			.setDesc(
				"Your VaultLink server's URL including the protocol and full path."
			)
			.setTooltip("This is the URL of the server you want to sync with.")
			.addText((text) =>
				text
					.setPlaceholder("https://example.com:3000")
					.setValue(this.editedServerUri)
					.onChange((value) => {
						this.editedServerUri = value;
						updateTitle(value);
					})
			);

		const [tokenTitle, updateTokenTitle] = this.unsavedAwareSettingName(
			"Access token",
			"token"
		);

		new Setting(containerEl)
			.setName(tokenTitle)
			.setClass("sync-settings-access-token")
			.setDesc(
				"Set the access token for the server that you can get from the server"
			)
			.setTooltip("todo, links to dcocs")
			.addTextArea((text) =>
				text
					.setPlaceholder("ey...")
					.setValue(this.editedToken)
					.onChange((value) => {
						this.editedToken = value;
						updateTokenTitle(value);
					})
			);

		const [vaultNameTitle, updateVaultNameTitle] =
			this.unsavedAwareSettingName("Vault name", "vaultName");

		new Setting(containerEl)
			.setName(vaultNameTitle)
			.setDesc(
				"Set the name of the remote vault that you want to sync with"
			)
			.setTooltip("todo, links to dcocs")
			.addText((text) =>
				text
					.setPlaceholder("My Obsidian Vault")
					.setValue(this.editedVaultName)
					.onChange((value) => {
						this.editedVaultName = value;
						updateVaultNameTitle(value);
					})
			);

		new Setting(containerEl)
			.addButton((button) =>
				button.setButtonText("Apply").onClick(async () => {
					if (
						this.editedVaultName !==
							this.syncClient.getSettings().vaultName ||
						this.editedServerUri !==
							this.syncClient.getSettings().remoteUri ||
						this.editedToken !== this.syncClient.getSettings().token
					) {
						await this.syncClient.setSettings({
							vaultName: this.editedVaultName,
							remoteUri: this.editedServerUri,
							token: this.editedToken
						});
						new Notice(
							"The changes have been applied successfully!"
						);
						await this.statusDescription.updateConnectionState();
					} else {
						new Notice("No changes to apply");
					}
				})
			)
			.addButton((button) =>
				button.setButtonText("Test connection").onClick(async () => {
					new Notice(
						(await this.syncClient.checkConnection()).message
					);
					await this.statusDescription.updateConnectionState();
				})
			);
	}

	private renderSyncSettings(containerEl: HTMLElement): void {
		containerEl.createEl("h3", { text: "Sync" });

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
					.setValue(this.syncClient.getSettings().isSyncEnabled)
					.onChange(async (value) =>
						this.syncClient.setSetting("isSyncEnabled", value)
					)
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
						this.syncClient.getSettings()
							.fetchChangesUpdateIntervalMs / 1000
					)
					.onChange(async (value) =>
						this.syncClient.setSetting(
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
					.setValue(this.syncClient.getSettings().syncConcurrency)
					.onChange(async (value) =>
						this.syncClient.setSetting("syncConcurrency", value)
					)
			);

		new Setting(containerEl)
			.setName("Maximum file size to be uploaded (MB)")
			.setDesc(
				"Set the maximum file size that can be uploaded to the server. Files larger than this size will be ignored."
			)
			.addSlider((slider) =>
				slider
					.setLimits(1, 64, 1)
					.setDynamicTooltip()
					.setInstant(false)
					.setValue(this.syncClient.getSettings().maxFileSizeMB)
					.onChange(async (value) =>
						this.syncClient.setSetting("maxFileSizeMB", value)
					)
			);

		new Setting(containerEl)
			.setName("Danger zone")
			.setDesc(
				"Delete the local metadata database while leaving the local and remote files intact."
			)
			.addButton((button) =>
				button.setButtonText("Reset sync state").onClick(async () => {
					await this.syncClient.reset();
					new Notice(
						"Sync state has been reset, you will need to resync"
					);
				})
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

	private unsavedAwareSettingName(
		name: string,
		settingName: keyof SyncSettings
	): [
		DocumentFragment,
		(newValue: SyncSettings[keyof SyncSettings]) => void
	] {
		const titleContainer = document.createDocumentFragment();
		const title = titleContainer.createEl("div", {
			text: name,
			cls: "setting-item-name"
		});

		const updateTitle = (
			currentValue: SyncSettings[keyof SyncSettings]
		): void => {
			title.innerText = `${name}${
				currentValue !== this.syncClient.getSettings()[settingName]
					? " (unsaved)"
					: ""
			}`;
		};

		return [titleContainer, updateTitle];
	}
}
