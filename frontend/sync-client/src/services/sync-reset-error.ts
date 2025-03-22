export class SyncResetError extends Error {
	public constructor() {
		super("Sync was reset");
		this.name = "SyncResetError";
	}
}
