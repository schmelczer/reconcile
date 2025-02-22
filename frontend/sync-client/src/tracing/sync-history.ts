import type { RelativePath } from "src/persistence/database";
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
	DELETE = "DELETE"
}

export enum SyncSource {
	PUSH = "PUSH",
	PULL = "PULL"
}

export enum SyncStatus {
	NO_OP = "NO_OP",
	SUCCESS = "SUCCESS",
	ERROR = "ERROR"
}

export type HistoryEntry = CommonHistoryEntry & { timestamp: Date };

export interface HistoryStats {
	success: number;
	error: number;
}

export class SyncHistory {
	private static readonly MAX_ENTRIES = 5000;

	private readonly entries: HistoryEntry[] = [];

	private readonly syncHistoryUpdateListeners: ((
		status: HistoryStats
	) => void)[] = [];

	private status: HistoryStats = {
		success: 0,
		error: 0
	};

	public constructor(private logger: Logger) {}

	public getEntries(): HistoryEntry[] {
		return [...this.entries];
	}

	public reset(): void {
		this.entries.length = 0;
		this.status = {
			success: 0,
			error: 0
		};
		this.syncHistoryUpdateListeners.forEach((listener) => {
			listener(this.status);
		});
	}

	public addSyncHistoryUpdateListener(
		listener: (stats: HistoryStats) => void
	): void {
		this.syncHistoryUpdateListeners.push(listener);
		listener({ ...this.status });
	}

	public addHistoryEntry(entry: CommonHistoryEntry): void {
		const historyEntry = {
			...entry,
			timestamp: new Date()
		};
		this.entries.push(historyEntry);

		if (entry.status === SyncStatus.SUCCESS) {
			this.status.success++;
			this.logger.info(
				`History entry: ${entry.relativePath} - ${entry.message}`
			);
		} else if (entry.status === SyncStatus.ERROR) {
			this.status.error++;
			this.logger.error(
				`Error syncing file: ${entry.relativePath} - ${entry.message}`
			);
		} else {
			this.logger.debug(
				`No-op syncing file: ${entry.relativePath} - ${entry.message}`
			);
		}

		this.syncHistoryUpdateListeners.forEach((listener) => {
			listener(this.status);
		});

		if (this.entries.length > SyncHistory.MAX_ENTRIES) {
			this.entries.shift();
		}
	}
}
