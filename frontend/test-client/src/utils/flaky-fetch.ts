import { sleep } from "./sleep";

export const flakyFetchFactory =
	(jitterScaleInSeconds: number) =>
	async (
		input: string | URL | globalThis.Request,
		init?: RequestInit
	): Promise<Response> => {
		if (jitterScaleInSeconds > 0) {
			await sleep(Math.random() * jitterScaleInSeconds * 1000);
		}

		const response = await fetch(input, init);

		if (jitterScaleInSeconds > 0) {
			await sleep(Math.random() * jitterScaleInSeconds * 1000);
		}

		return response;
	};
