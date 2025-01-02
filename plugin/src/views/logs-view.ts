import type { WorkspaceLeaf } from "obsidian";
import { ItemView } from "obsidian";
import { LogLevel, Logger } from "src/tracing/logger";

export class LogsView extends ItemView {
	public static readonly TYPE = "logs-view";
	public static readonly ICON = "logs";

	public constructor(leaf: WorkspaceLeaf) {
		super(leaf);
		this.icon = LogsView.ICON;
		Logger.getInstance().addOnMessageListener(() => {
			this.updateView();
		});
	}

	private static formatTimestamp(timestamp: Date): string {
		return timestamp.toTimeString().split(" ")[0];
	}

	public getViewType(): string {
		return LogsView.TYPE;
	}

	public getDisplayText(): string {
		return "VaultLink logs";
	}

	public async onOpen(): Promise<void> {
		this.updateView();
	}

	private updateView(): void {
		const container = this.containerEl.children[1];
		container.empty();

		container.createEl("h4", { text: "VaultLink logs" });

		Logger.getInstance()
			.getMessages(LogLevel.DEBUG)
			.forEach((message) => {
				const messageContainer = container.createDiv({
					cls: ["log-message", message.level],
				});
				messageContainer.createEl("span", {
					text: ` | ${LogsView.formatTimestamp(
						message.timestamp
					)} | `,
				});
				messageContainer.createEl("span", { text: message.message });
			});
	}
}
