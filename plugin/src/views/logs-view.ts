import type { WorkspaceLeaf } from "obsidian";
import { ItemView } from "obsidian";
import { LogLevel, Logger } from "src/tracing/logger";

export class LogsView extends ItemView {
	public static readonly TYPE = "logs-view";
	public static readonly ICON = "logs";

	private timer: NodeJS.Timer | null = null;

	public constructor(leaf: WorkspaceLeaf) {
		super(leaf);
		this.icon = LogsView.ICON;
	}

	public getViewType(): string {
		return LogsView.TYPE;
	}

	public getDisplayText(): string {
		return "VaultLink logs";
	}

	public async onOpen(): Promise<void> {
		// eslint-disable-next-line @typescript-eslint/no-misused-promises
		this.timer = setInterval(async () => this.updateView(), 250);
	}

	public async onClose(): Promise<void> {
		if (this.timer) {
			clearInterval(this.timer);
			this.timer = null;
		}
	}

	private async updateView(): Promise<void> {
		const container = this.containerEl.children[1];
		container.empty();

		container.createEl("h4", { text: "VaultLink logs" });

		const messages = Logger.getInstance()
			.getMessages(LogLevel.DEBUG)
			.map((message) => message.toString())
			.join("\n");

		container.createEl("pre", { text: messages });
	}
}
