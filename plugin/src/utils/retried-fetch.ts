import * as fetchRetryFactory from "fetch-retry";
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

export async function retriedFetch(
	input: RequestInfo | URL,
	init: RequestInit = {}
): Promise<Response> {
	return fetchWithRetry(input, {
		...init,
		retryOn: function (attempt, error, response) {
			if (error !== null || !response || response.status >= 500) {
				Logger.getInstance().warn(
					`Retrying fetch for ${getUrlFromInput(
						input
					)}, attempt ${attempt}`
				);

				return true;
			}
			return false;
		},
		retries: 6,
		retryDelay: (attempt) => Math.pow(1.5, attempt) * 500,
	});
}
