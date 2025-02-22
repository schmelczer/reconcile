import { choose } from "../utils/choose";
import { v4 as uuidv4 } from "uuid";
import { assert } from "../utils/assert";
import type { SyncSettings } from "sync-client";
import { LogLevel } from "sync-client";
import { MockClient } from "./mock-client";
import chalk from "chalk";
import { sleep } from "../utils/sleep";

export class MockAgent extends MockClient {
	private readonly writtenContents: string[] = [];
	private readonly pendingActions: Promise<unknown>[] = [];

	public constructor(
		globalFiles: Record<string, Uint8Array>,
		initialSettings: Partial<SyncSettings>,
		public readonly name: string,
		private readonly color: string,
		private readonly doDeletes: boolean
	) {
		super(globalFiles, initialSettings);
	}

	public async init(): Promise<void> {
		await super.init();

		this.client.logger.addOnMessageListener((message) => {
			const formatted = chalk.hex(this.color)(
				`[${this.name}] ${message.timestamp.toISOString()} ${message.level} ${message.message}`
			);

			switch (message.level) {
				case LogLevel.ERROR:
					console.error(formatted);
					break;
				case LogLevel.WARNING:
					console.warn(formatted);
					break;
				case LogLevel.INFO:
					console.info(formatted);
					break;
				case LogLevel.DEBUG:
					console.debug(formatted);
					break;
			}
		});

		this.client.logger.info("Agent initialized");
	}

	public async act(): Promise<void> {
		const options: (() => Promise<unknown>)[] = [
			async (): Promise<unknown> => {
				const file = this.getFileName();
				this.client.logger.info(`Decided to create file ${file}`);
				return this.create(
					file,
					new TextEncoder().encode(this.getContent())
				);
			},
			async (): Promise<unknown> => {
				this.client.logger.info(
					`Decided to change fetchChangesUpdateIntervalMs`
				);
				return this.client.settings.setSetting(
					"fetchChangesUpdateIntervalMs",
					Math.random() * 1000
				);
			},
			async (): Promise<unknown> => {
				this.client.logger.info(`Decided to disable sync`);
				return this.client.settings.setSetting("isSyncEnabled", false);
			},
			async (): Promise<unknown> => {
				this.client.logger.info(`Decided to enable sync`);
				return this.client.settings.setSetting("isSyncEnabled", true);
			}
		];

		const files = await this.listAllFiles();

		if (files.length > 0) {
			options.push(
				async (): Promise<unknown> => {
					const file = choose(files);

					const newName = this.getFileName();
					this.client.logger.info(
						`Decided to rename file ${file} to ${newName}`
					);
					return this.rename(file, newName);
				},
				async (): Promise<unknown> => {
					const file = choose(files);

					this.client.logger.info(`Decided to update file ${file}`);
					return this.atomicUpdateText(
						file,
						(old) => old + " " + this.getContent()
					);
				}
			);

			if (this.doDeletes) {
				options.push(async () => this.delete(choose(files)));
			}
		}

		this.pendingActions.push(choose(options)());
	}

	public async finish(): Promise<void> {
		await Promise.all(this.pendingActions);
		await this.client.settings.setSetting("isSyncEnabled", true);
		await this.client.syncer.applyRemoteChangesLocally();
		await sleep(5000);
		await this.client.syncer.waitForSyncQueue();
		this.client.stop();
	}

	public assertFileSystemIsConsistent(): void {
		const globalFiles = Object.keys(this.globalFiles);
		const localFiles = Object.keys(this.localFiles);

		const missingInGlobal = localFiles.filter(
			(file) => !(file in this.globalFiles)
		);
		const missingInLocal = globalFiles.filter(
			(file) => !(file in this.localFiles)
		);

		assert(
			missingInGlobal.length === 0,
			`Files missing in global files: ${missingInGlobal.join(", ")}`
		);
		assert(
			missingInLocal.length === 0,
			`Files missing in local files: ${missingInLocal.join(", ")}`
		);

		for (const file of globalFiles) {
			const localContent = new TextDecoder().decode(
				this.localFiles[file]
			);
			const globalContent = new TextDecoder().decode(
				this.globalFiles[file]
			);
			assert(
				localContent === globalContent,
				`Content mismatch for file ${file}: ${localContent} <> ${globalContent}`
			);
		}
	}

	public assertAllContentIsPresentOnce(): void {
		for (const content of this.writtenContents) {
			const found = Object.values(this.localFiles).filter((file) => {
				return new TextDecoder().decode(file).includes(content);
			});

			if (this.doDeletes) {
				assert(
					found.length <= 1,
					`Content ${content} found in ${found.length} files`
				);
			} else {
				assert(
					found.length === 1,
					`Content ${content} found in ${found.length} files`
				);

				const [file] = found;
				assert(
					new TextDecoder().decode(file).split(content).length === 2,
					`Content ${content} found more than once in a file`
				);
			}
		}
	}

	private getContent(): string {
		const uuid = uuidv4();
		this.writtenContents.push(uuid);
		return uuid;
	}

	private getFileName(): string {
		return `${this.name}-${uuidv4()}.md`;
	}
}
