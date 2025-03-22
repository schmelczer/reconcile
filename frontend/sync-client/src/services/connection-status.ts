import type { Settings } from "../persistence/settings";
import type { Logger } from "../tracing/logger";
import { createPromise } from "../utils/create-promise";
import { sleep } from "../utils/sleep";

export class ConnectionStatus {
	private static readonly UNTIL_RESOLUTION = Symbol();
	private canFetch: boolean;
	private until: Promise<symbol>;
	private resolveUntil: (result: symbol) => void;
	private rejectUntil: (reason: unknown) => void;

	public constructor(
		settings: Settings,
		private readonly logger: Logger
	) {
		this.canFetch = settings.getSettings().isSyncEnabled;

		[this.until, this.resolveUntil, this.rejectUntil] =
			createPromise<symbol>();

		settings.addOnSettingsChangeListener((newSettings, oldSettings) => {
			if (oldSettings.isSyncEnabled != newSettings.isSyncEnabled) {
				this.canFetch = newSettings.isSyncEnabled;
				this.resolveUntil(ConnectionStatus.UNTIL_RESOLUTION);
				[this.until, this.resolveUntil, this.rejectUntil] =
					createPromise<symbol>();
			}
		});
	}

	private static getUrlFromInput(input: RequestInfo | URL): string {
		if (input instanceof URL) {
			return input.href;
		}
		if (typeof input === "string") {
			return input;
		}
		return input.url;
	}

	public reset(): void {
		this.rejectUntil(new Error("Sync was reset"));
		[this.until, this.resolveUntil, this.rejectUntil] = createPromise();
	}

	public getFetchImplementation(
		logger: Logger,
		fetch: typeof globalThis.fetch = globalThis.fetch
	): typeof globalThis.fetch {
		return async (input: RequestInfo | URL): Promise<Response> => {
			while (!this.canFetch) {
				await this.until;
			}

			try {
				// https://github.com/jonbern/fetch-retry/blob/8684ef4e688375f623bd76f13add76dbc1d67cfb/index.js#L67C1-L70C21
				const _input =
					typeof Request !== "undefined" && input instanceof Request
						? input.clone()
						: input;

				const fetchPromise = fetch(_input);

				// We only want to catch rejections from `this.until`
				let result: symbol | Response | undefined = undefined;
				do {
					result = await Promise.race([this.until, fetchPromise]);
				} while (result === ConnectionStatus.UNTIL_RESOLUTION);

				const fetchResult: Response = result as Response; // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion

				if (!fetchResult.ok) {
					this.logger.warn(
						`Fetch for ${ConnectionStatus.getUrlFromInput(
							input
						)}, got status ${fetchResult.status}`
					);
				}

				return fetchResult;
			} catch (error) {
				logger.warn(
					`Fetch for ${ConnectionStatus.getUrlFromInput(
						input
					)}, got error: ${error}`
				);
				throw error;
			}
		};
	}
}
