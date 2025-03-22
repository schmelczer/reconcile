import type { RelativePath } from "../persistence/database";
import type { Logger } from "./logger";

export interface CommonHistoryEntry {
	status: SyncStatus;
	relativePath: RelativePath;
	message: string;
	type?: SyncType;
}

export enum SyncType {
	CREATE = "CREATE",
	UPDATE = "UPDATE",
	DELETE = "DELETE"
}

export enum SyncStatus {
	SUCCESS = "SUCCESS",
	ERROR = "ERROR"
}

export type HistoryEntry = CommonHistoryEntry & { timestamp: Date };

export interface HistoryStats {
	success: number;
	error: number;
}

export class SyncHistory {
	private static readonly MAX_ENTRIES = 500;

	private entries: HistoryEntry[] = [];

	private readonly syncHistoryUpdateListeners: ((
		status: HistoryStats
	) => void)[] = [];

	private status: HistoryStats = {
		success: 0,
		error: 0
	};

	public constructor(private readonly logger: Logger) {}

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

		const candidate = this.entries.find(
			(e) => e.relativePath === historyEntry.relativePath
		);
		if (
			candidate !== undefined &&
			(this.entries.slice(-1)[0] === candidate ||
				candidate.timestamp.getTime() + 10 * 1000 >
					historyEntry.timestamp.getTime())
		) {
			this.entries = this.entries.filter((e) => e !== candidate);
		}
		this.entries.push(historyEntry);

		if (entry.status === SyncStatus.SUCCESS) {
			this.status.success++;
			this.logger.info(
				`History entry: ${entry.relativePath} - ${entry.message}`
			);
		} else {
			this.status.error++;
			this.logger.error(
				`Error syncing file: ${entry.relativePath} - ${entry.message}`
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
