import * as fetchRetryFactory from "fetch-retry";
import { Logger } from "src/tracing/logger";

const fetchWithRetry = fetchRetryFactory.default(fetch);

export async function retriedFetch(
	input: RequestInfo | URL,
	init: RequestInit = {}
): Promise<Response> {
	return fetchWithRetry(input, {
		...init,
		retryOn: function (attempt, error, response) {
			// retry on any network error, or 4xx or 5xx status codes
			if (error !== null || !response || response.status >= 500) {
				Logger.getInstance().warn(
					`Retrying fetch attempt ${attempt} for ${getUrlFromInput(
						input
					)}`
				);

				return true;
			}
			return false;
		},
		retries: 6,
		retryDelay: function (attempt) {
			Logger;
			return Math.pow(1.5, attempt) * 500;
		},
	});
}

function getUrlFromInput(input: RequestInfo | URL): string {
	if (input instanceof URL) {
		return input.href;
	}
	if (typeof input === "string") {
		return input;
	}
	return input.url;
}
