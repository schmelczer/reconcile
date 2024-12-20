import type { RelativePath } from "src/database/document-metadata";
import { Logger } from "./logger";

export interface CommonHistoryEntry {
	status: SyncStatus;
	relativePath: RelativePath;
	message: string;
	type?: SyncType;
	source?: SyncSource;
}

export enum SyncType {
	CREATE = "CREATE",
	UPDATE = "UPDATE",
	DELETE = "DELETE",
}

export enum SyncSource {
	PUSH = "PUSH",
	PULL = "PULL",
}

export enum SyncStatus {
	NO_OP = "NO_OP",
	SUCCESS = "SUCCESS",
	ERROR = "ERROR",
}

export type HistoryEntry = CommonHistoryEntry & { timestamp: Date };

export interface HistoryStats {
	success: number;
	error: number;
}

export class SyncHistory {
	private static readonly MAX_ENTRIES = 1000;

	private entries: HistoryEntry[] = [];
	private readonly requestCountListeners: ((status: HistoryStats) => void)[] =
		[];
	private status: HistoryStats = {
		success: 0,
		error: 0,
	};

	public getMessages(): HistoryEntry[] {
		return this.entries;
	}

	public reset(): void {
		this.entries = [];
		this.status = {
			success: 0,
			error: 0,
		};
		this.requestCountListeners.forEach((listener) => {
			listener(this.status);
		});
	}

	public addSyncHistoryStatsChangeListener(
		listener: (status: HistoryStats) => void
	): void {
		this.requestCountListeners.push(listener);
		listener({ ...this.status });
	}

	public addHistoryEntry(entry: CommonHistoryEntry): void {
		const historyEntry = {
			...entry,
			timestamp: new Date(),
		};
		this.entries.push(historyEntry);

		if (entry.status === SyncStatus.SUCCESS) {
			this.status.success++;
			Logger.getInstance().info(`Synced file: ${entry.relativePath}`);
		} else if (entry.status === SyncStatus.ERROR) {
			this.status.error++;
			Logger.getInstance().error(
				`Error syncing file: ${entry.relativePath} - ${entry.message}`
			);
		}

		this.requestCountListeners.forEach((listener) => {
			listener(this.status);
		});

		if (this.entries.length > SyncHistory.MAX_ENTRIES) {
			this.entries.shift();
		}
	}
}
