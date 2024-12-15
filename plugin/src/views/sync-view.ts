import { ItemView, WorkspaceLeaf } from "obsidian";
import { Logger } from "src/logger";

export class SyncView extends ItemView {
	public static TYPE = "example-view";

	public constructor(leaf: WorkspaceLeaf) {
		super(leaf);
	}

	getViewType() {
		return SyncView.TYPE;
	}

	getDisplayText() {
		return "Example view";
	}

	async onOpen() {
		const container = this.containerEl.children[1];
		container.empty();
		container.createEl("h4", { text: "Example view" });

		setInterval(() => this.updateView(), 1000);
	}

	async updateView() {
		const container = this.containerEl.children[1];
		container.empty();

		const messages = Logger.getInstance()
			.getMessages()
			.map((message) => message.toString())
			.join("\n");

		container.createEl("pre", { text: messages });
	}

	async onClose() {
		// Nothing to clean up.
	}
}
