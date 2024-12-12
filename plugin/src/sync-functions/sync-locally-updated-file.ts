import { TFile } from "obsidian";
import { Database } from "src/database/database";
import { SyncServer } from "src/services/sync_service";
import { hash } from "src/utils";
import { syncLocallyRenamedFile } from "./sync-locally-renamed-file";

export async function syncLocallyUpdatedFile(
	database: Database,
	syncServer: SyncServer,
	file: TFile
) {
	syncLocallyRenamedFile(database, syncServer, file, file.path);
}
