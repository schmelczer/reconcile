import type { WorkspaceLeaf } from "obsidian";
import { ItemView } from "obsidian";
import type { LogLine } from "sync-client";
import { LogLevel, type SyncClient } from "sync-client";

export class LogsView extends ItemView {
	public static readonly TYPE = "logs-view";
	public static readonly ICON = "logs";

	private logsContainer: HTMLElement | undefined;
	private readonly logLineToElement = new Map<LogLine, HTMLElement>();

	public constructor(
		private readonly client: SyncClient,
		leaf: WorkspaceLeaf
	) {
		super(leaf);
		this.icon = LogsView.ICON;
		this.client.logger.addOnMessageListener(() => {
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

		const container = this.containerEl.children[1];
		container.addClass("logs-view");
		container.createEl("h4", { text: "VaultLink logs" });
		this.logsContainer = container.createDiv({ cls: "logs-container" });
	}

	private updateView(): void {
		const container = this.logsContainer;
		if (container === undefined) {
			return;
		}

		const logs = this.client.logger.getMessages(LogLevel.DEBUG);

		if (this.logLineToElement.size === 0 && logs.length > 0) {
			// Clear the "No logs available yet" message
			container.empty();
		}

		logs.forEach((message) => {
			if (this.logLineToElement.has(message)) {
				return;
			}

			const element = container.createDiv(
				{
					cls: ["log-message", message.level]
				},
				(messageContainer) => {
					messageContainer.createEl("span", {
						text: LogsView.formatTimestamp(message.timestamp),
						cls: "timestamp"
					});
					messageContainer.createEl("span", {
						text: message.message
					});
				}
			);

			this.logLineToElement.set(message, element);
		});

		const newLines = new Set(logs);
		for (const [logLine, element] of this.logLineToElement) {
			if (!newLines.has(logLine)) {
				element.remove();
				this.logLineToElement.delete(logLine);
			}
		}

		if (logs.length === 0) {
			container.createEl("p", {
				text: "No logs available yet."
			});
		}
	}
}
