import "./logs-view.scss";

import type { WorkspaceLeaf } from "obsidian";
import { ItemView } from "obsidian";
import type { LogLine } from "sync-client";
import { LogLevel, type SyncClient } from "sync-client";

export class LogsView extends ItemView {
	public static readonly TYPE = "logs-view";
	public static readonly ICON = "logs";

	private static readonly MAX_OFFSET_FROM_BOTTOM_WITH_AUTO_SCROLL_PX = 300;

	private logsContainer: HTMLElement | undefined;
	private readonly logLineToElement = new Map<LogLine, HTMLElement>();
	private minLogLevel: LogLevel = LogLevel.INFO;

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

	private static createLogLineElement(
		container: HTMLElement,
		logLine: LogLine
	): HTMLElement {
		return container.createDiv(
			{
				cls: ["log-message", logLine.level]
			},
			(messageContainer) => {
				messageContainer.createEl("span", {
					text: LogsView.formatTimestamp(logLine.timestamp),
					cls: "timestamp"
				});
				messageContainer.createEl("span", {
					text: logLine.message
				});
			}
		);
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
		const container = this.containerEl.children[1];
		container.addClass("logs-view");

		const logLevels = [
			{ label: "Debug", value: LogLevel.DEBUG },
			{ label: "Info", value: LogLevel.INFO },
			{ label: "Warn", value: LogLevel.WARNING },
			{ label: "Error", value: LogLevel.ERROR }
		];

		container.createDiv(
			{
				cls: "verbosity-selector"
			},
			(verbositySection) => {
				verbositySection.createEl("h4", {
					text: "VaultLink logs"
				});

				verbositySection.createEl("select", {}, (dropdown) => {
					logLevels.forEach(({ label, value }) =>
						dropdown.createEl("option", { text: label, value })
					);

					dropdown.value = this.minLogLevel;

					dropdown.addEventListener("change", () => {
						// eslint-disable-next-line @typescript-eslint/no-unsafe-type-assertion
						this.minLogLevel = dropdown.value as LogLevel;

						this.logsContainer?.empty();
						this.logLineToElement.clear();
						this.updateView();
					});
				});
			}
		);

		this.logsContainer = container.createDiv({ cls: "logs-container" });
	}

	private updateView(): void {
		const container = this.logsContainer;
		if (container === undefined) {
			return;
		}

		const logs = this.client.logger.getMessages(this.minLogLevel);

		if (this.logLineToElement.size === 0 && logs.length > 0) {
			// Clear the "No logs available yet" message
			container.empty();
		}

		const shouldScroll =
			container.scrollTop == 0 ||
			container.scrollHeight -
				container.clientHeight -
				container.scrollTop <
				LogsView.MAX_OFFSET_FROM_BOTTOM_WITH_AUTO_SCROLL_PX;

		logs.forEach((message) => {
			if (this.logLineToElement.has(message)) {
				return;
			}

			const element = LogsView.createLogLineElement(container, message);

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
			container.empty();
			container.createEl("p", {
				text: "No logs available yet."
			});
		} else if (shouldScroll) {
			container.scrollTop = container.scrollHeight;
		}
	}
}
