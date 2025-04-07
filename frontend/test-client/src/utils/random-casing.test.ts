import { randomCasing } from "./random-casing";

describe("randomCasing", () => {
	it("simple test", () => {
		const input =
			"hello, this is a really long string with a lot of characters";
		const result = randomCasing(input);
		expect(result.toLowerCase()).toBe(input.toLowerCase());
		expect(result).not.toBe(input);
	});
});
