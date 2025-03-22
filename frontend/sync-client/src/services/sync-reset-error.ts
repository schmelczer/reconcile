export class SyncResetError extends Error {
	constructor() {
		super("Sync was reset");
		this.name = "SyncResetError";
	}
}
