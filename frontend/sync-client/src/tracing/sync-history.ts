import type { RelativePath } from "../persistence/database";
import type { Logger } from "./logger";

export interface SyncCreateDetails {
	type: SyncType.CREATE;
	relativePath: RelativePath;
}

export interface SyncUpdateDetails {
	type: SyncType.UPDATE;
	relativePath: RelativePath;
}

export interface SyncMovedDetails {
	type: SyncType.MOVE;
	relativePath: RelativePath;
	movedFrom: RelativePath;
}

export interface SyncDeleteDetails {
	type: SyncType.DELETE;
	relativePath: RelativePath;
}

export interface SyncSkippedDetails {
	type: SyncType.SKIPPED;
	relativePath: RelativePath;
}

export type SyncDetails =
	| SyncCreateDetails
	| SyncUpdateDetails
	| SyncDeleteDetails
	| SyncMovedDetails
	| SyncSkippedDetails;

export interface CommonHistoryEntry {
	status: SyncStatus;
	message: string;
	details: SyncDetails;
	author?: string;
}

export enum SyncType {
	CREATE = "CREATE",
	UPDATE = "UPDATE",
	DELETE = "DELETE",
	MOVE = "MOVE",
	SKIPPED = "SKIPPED"
}

export enum SyncStatus {
	SUCCESS = "SUCCESS",
	ERROR = "ERROR",
	SKIPPED = "SKIPPED"
}

export type HistoryEntry = CommonHistoryEntry & { timestamp: Date };

export interface HistoryStats {
	success: number;
	error: number;
}

export class SyncHistory {
	private static readonly MAX_ENTRIES = 5000;
	private static readonly TIMEOUT_FOR_MERGING_ENTRIES_IN_SECONDS = 60;

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

		const candidate = this.findSimilarRecentUpdateEntry(historyEntry);
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

	private findSimilarRecentUpdateEntry(
		entry: HistoryEntry
	): HistoryEntry | undefined {
		if (entry.details.type !== SyncType.UPDATE) {
			return;
		}

		const candidate = this._entries.find(
			(e) =>
				e.details.type === SyncType.UPDATE &&
				e.details.relativePath === entry.details.relativePath
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
		const message = `${entry.details.relativePath} - ${entry.message} (${entry.details.type.toLocaleLowerCase()})`;
		switch (entry.status) {
			case SyncStatus.SUCCESS:
				this.status.success++;
				this.logger.info(`History entry: ${message}`);
				break;
			case SyncStatus.ERROR:
				this.status.error++;
				this.logger.error(`Cannot sync file: ${message}`);
				break;
			case SyncStatus.SKIPPED:
				this.logger.error(`Skipping file: ${message}`);
				break;
		}

		this.syncHistoryUpdateListeners.forEach((listener) => {
			listener(this.status);
		});
	}
}
