import type { WorkspaceLeaf } from "obsidian";
import { ItemView } from "obsidian";
import { LogLevel, Logger } from "src/tracing/logger";

export class SyncView extends ItemView {
	public static readonly TYPE = "example-view";

	public constructor(leaf: WorkspaceLeaf) {
		super(leaf);
	}

	public getViewType(): string {
		return SyncView.TYPE;
	}

	public getDisplayText(): string {
		return "Example view";
	}

	public async onOpen(): Promise<void> {
		const container = this.containerEl.children[1];
		container.empty();
		container.createEl("h4", { text: "Example view" });

		// eslint-disable-next-line @typescript-eslint/no-misused-promises
		setInterval(async () => this.updateView(), 1000);
	}

	public async updateView(): Promise<void> {
		const container = this.containerEl.children[1];
		container.empty();

		const messages = Logger.getInstance()
			.getMessages(LogLevel.DEBUG)
			.map((message) => message.toString())
			.join("\n");

		container.createEl("pre", { text: messages });
	}
}
