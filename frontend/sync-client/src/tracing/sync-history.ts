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
	private static readonly TIMEOUT_FOR_MERGING_ENTRIES_IN_SECONDS = 15;

	private _entries: HistoryEntry[] = [];

	private readonly syncHistoryUpdateListeners: ((
		status: HistoryStats
	) => void)[] = [];

	private status: HistoryStats = {
		success: 0,
		error: 0
	};

	public constructor(private readonly logger: Logger) {}

	public get entries(): readonly HistoryEntry[] {
		return this._entries;
	}

	/**
	 * Insert the entry at the beginning of the history list. If the entry
	 * already in the list, it will get moved to the beginning and updated.
	 *
	 * If the entry list is too long, the oldest entry will be removed.
	 */
	public addHistoryEntry(entry: CommonHistoryEntry): void {
		const historyEntry = {
			...entry,
			timestamp: new Date()
		};

		const candidate = this.findSimilarRecentEntry(historyEntry);
		if (candidate !== undefined) {
			this._entries = this._entries.filter((e) => e !== candidate);
		}

		// Insert the entry at the beginning
		this._entries.unshift(historyEntry);

		if (this._entries.length > SyncHistory.MAX_ENTRIES) {
			this._entries.pop();
		}

		this.updateSuccessCount(historyEntry);
	}

	public addSyncHistoryUpdateListener(
		listener: (stats: HistoryStats) => void
	): void {
		this.syncHistoryUpdateListeners.push(listener);
		listener({ ...this.status });
	}

	public reset(): void {
		this._entries.length = 0;
		this.status = {
			success: 0,
			error: 0
		};
		this.syncHistoryUpdateListeners.forEach((listener) => {
			listener(this.status);
		});
	}

	private findSimilarRecentEntry(
		entry: HistoryEntry
	): HistoryEntry | undefined {
		const candidate = this._entries.find(
			(e) => e.relativePath === entry.relativePath
		);
		if (
			candidate !== undefined &&
			(this._entries[0] === candidate ||
				candidate.timestamp.getTime() +
					SyncHistory.TIMEOUT_FOR_MERGING_ENTRIES_IN_SECONDS * 1000 >
					entry.timestamp.getTime())
		) {
			return candidate;
		}
	}

	private updateSuccessCount(entry: HistoryEntry): void {
		if (entry.status === SyncStatus.SUCCESS) {
			this.status.success++;
			this.logger.info(
				`History entry: ${entry.relativePath} - ${entry.message}`
			);
		} else {
			this.status.error++;
			this.logger.error(
				`Cannot sync file: ${entry.relativePath} - ${entry.message}`
			);
		}
		this.syncHistoryUpdateListeners.forEach((listener) => {
			listener(this.status);
		});
	}
}
