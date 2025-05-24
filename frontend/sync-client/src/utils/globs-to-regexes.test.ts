import { Logger } from "../tracing/logger";
import { globsToRegex } from "./globs-to-regexes";

describe("globsToRegexes", () => {
	it("basicExample", async () => {
		const regex = globsToRegex([".git/**"], new Logger())[0];

		expect(regex.test(".git/objects/object")).toBeTruthy();
	});
});
