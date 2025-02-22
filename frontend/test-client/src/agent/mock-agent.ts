import { choose } from "../utils/choose";
import { v4 as uuidv4 } from "uuid";
import { assert } from "../utils/assert";
import { SyncSettings } from "sync-client";
import { MockClient } from "./mock-client";

export class MockAgent extends MockClient {
	private writtenContents: Array<string> = [];
	private pendingActions: Array<Promise<unknown>> = [];

	public constructor(
		globalFiles: Record<string, Uint8Array>,
		initialSettings: Partial<SyncSettings>,
		private readonly name: string
	) {
		super(globalFiles, initialSettings);
	}

	public async act(): Promise<void> {
		let options: Array<() => Promise<unknown>> = [
			() =>
				this.create(
					this.getFileName(),
					new TextEncoder().encode(this.getContent())
				),
			() =>
				this.client.settings.setSetting(
					"fetchChangesUpdateIntervalMs",
					Math.random() * 1000
				),
			() => this.client.settings.setSetting("isSyncEnabled", false),
			() => this.client.settings.setSetting("isSyncEnabled", true)
		];

		let files = await this.listAllFiles();

		if (files.length > 0) {
			options.push(
				() => this.rename(choose(files), this.getFileName()),
				() =>
					this.atomicUpdateText(
						choose(files),
						(old) => old + " " + this.getContent()
					)
			);
		}

		this.pendingActions.push(choose(options)());
	}

	private getContent() {
		const uuid = uuidv4();
		this.writtenContents.push(uuid);
		return uuid;
	}

	private getFileName() {
		return `${this.name}-${uuidv4()}.md`;
	}

	public async finish(): Promise<void> {
		await Promise.all(this.pendingActions);
		await this.client.settings.setSetting("isSyncEnabled", true);
		await this.client.syncer.applyRemoteChangesLocally();
	}

	public assertFileSystemIsConsistent(): void {
		const files = Object.keys(this.globalFiles);
		const localFiles = Object.keys(this.files);

		assert(
			files.length === localFiles.length,
			`File count mismatch: ${files.length} != ${localFiles.length}`
		);

		for (const file of files) {
			assert(
				file in this.globalFiles,
				`File ${file} missing in global files`
			);
			assert(
				new TextDecoder().decode(this.globalFiles[file]) ===
					new TextDecoder().decode(this.files[file]),
				`File ${file} content mismatch`
			);
		}
	}

	public assertAllContentIsPresentOnce(): void {
		for (const content of this.writtenContents) {
			const found = Object.values(this.files).filter((file) => {
				return new TextDecoder().decode(file).includes(content);
			});

			assert(
				found.length === 1,
				`Content ${content} found in ${found.length} files`
			);

			const file = found[0];
			assert(
				new TextDecoder().decode(file).split(content).length === 2,
				`Content ${content} found more than once in a file`
			);
		}
	}
}
