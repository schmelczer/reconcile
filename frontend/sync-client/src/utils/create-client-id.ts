import { v4 as uuidv4 } from "uuid";

export function createClientId(): string {
	// @ts-expect-error, injected by webpack
	const packageVersion = __CURRENT_VERSION__; // eslint-disable-line

	const platform =
		typeof navigator !== "undefined"
			? navigator.platform // eslint-disable-line @typescript-eslint/no-deprecated
			: typeof process !== "undefined"
				? process.platform
				: "unknown";

	return `vault-link/${packageVersion} (${uuidv4()}; ${platform})`;
}
