import * as fetchRetryFactory from "fetch-retry";
import type { RequestInitRetryParams } from "fetch-retry";
import { Logger } from "src/tracing/logger";

const fetchWithRetry = fetchRetryFactory.default(fetch);

function getUrlFromInput(input: RequestInfo | URL): string {
	if (input instanceof URL) {
		return input.href;
	}
	if (typeof input === "string") {
		return input;
	}
	return input.url;
}

export function retriedFetchFactory(logger: Logger) {
	return (
		input: RequestInfo | URL,
		init: RequestInitRetryParams<typeof fetch> = {}
	): Promise<Response> => {
		return fetchWithRetry(input, {
			retryOn: function (attempt, error, response) {
				if (error !== null || !response || response.status >= 500) {
					logger.warn(
						`Retrying fetch for ${getUrlFromInput(input)}, attempt ${attempt}`
					);

					return true;
				}
				return false;
			},
			retries: 6,
			retryDelay: (attempt) => Math.pow(1.5, attempt) * 500,
			...init
		});
	};
}
