import type { Settings } from "../persistence/settings";
import type { Logger } from "../tracing/logger";
import { createPromise } from "../utils/create-promise";
import { retriedFetchFactory } from "../utils/retried-fetch";

export class ConnectedState {
	private resolveIsSyncEnabled: (() => void) | undefined;
	private syncIsEnabled: Promise<void> | undefined;

	public constructor(
		settings: Settings,
		private readonly logger: Logger
	) {
		settings.addOnSettingsChangeHandlers((newSettings, oldSettings) => {
			if (!oldSettings.isSyncEnabled && newSettings.isSyncEnabled) {
				this.handleComingOnline();
			} else if (
				oldSettings.isSyncEnabled &&
				!newSettings.isSyncEnabled
			) {
				this.handleGoingOffline();
			}
		});
	}

	public getFetchImplementation(
		fetch: typeof globalThis.fetch,
		{ doRetries = true }: { doRetries: boolean } = { doRetries: true }
	): typeof globalThis.fetch {
		const retriedFetch = doRetries
			? retriedFetchFactory(this.logger, fetch)
			: fetch;

		return async (input: RequestInfo | URL): Promise<Response> => {
			if (this.syncIsEnabled !== undefined) {
				await this.syncIsEnabled;
			}
			return retriedFetch(input);
		};
	}

	private handleComingOnline(): void {
		this.logger.debug("Sync is enabled");
		this.resolveIsSyncEnabled?.();
	}

	private handleGoingOffline(): void {
		this.logger.debug("Sync is disabled");
		[this.syncIsEnabled, this.resolveIsSyncEnabled] = createPromise();
	}
}
