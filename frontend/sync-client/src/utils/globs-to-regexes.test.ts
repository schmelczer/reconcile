import { Logger } from "../tracing/logger";
import { globsToRegexes } from "./globs-to-regexes";

describe("globsToRegexes", () => {
	it("basicExample", async () => {
		const regex = globsToRegexes([".git/**"], new Logger())[0];

		expect(regex.test(".git/objects/object")).toBeTruthy();
		expect(regex.test(".git/objects/.object")).toBeTruthy();
	});
});
