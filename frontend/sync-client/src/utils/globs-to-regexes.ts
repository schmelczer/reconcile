import { makeRe } from "minimatch";
import type { Logger } from "../tracing/logger";

export function globsToRegexes(globs: string[], logger: Logger): RegExp[] {
	return globs
		.map((pattern) => {
			const result = makeRe(pattern, {
				dot: true
			});
			if (result === false) {
				logger.warn(
					`Failed to parse ${pattern}' as a glob pattern, skipping it`
				);
			}
			return result;
		})
		.filter((pattern) => pattern !== false);
}
