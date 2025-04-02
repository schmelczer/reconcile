import { lineAndColumnToPosition } from "./line-and-column-to-position";

describe("lineAndColumnToPosition", () => {
	it("should return the correct position for the first line", () => {
		const text = "Hello\nWorld";
		const position = lineAndColumnToPosition(text, 0, 3);
		expect(position).toBe(3);
	});

	it("should return the correct position for the second line", () => {
		const text = "Hello\nWorld";
		const position = lineAndColumnToPosition(text, 1, 2);
		expect(position).toBe(8);
	});

	it("should return the correct position for an empty string", () => {
		const text = "";
		const position = lineAndColumnToPosition(text, 0, 0);
		expect(position).toBe(0);
	});

	it("should handle a single-line string correctly", () => {
		const text = "SingleLine";
		const position = lineAndColumnToPosition(text, 0, 5);
		expect(position).toBe(5);
	});

	it("should handle multi-line strings with varying lengths", () => {
		const text = "Line1\nLongerLine2\nShort3";
		const position = lineAndColumnToPosition(text, 2, 4);
		expect(position).toBe(22);
	});

	it("should throw an error if the line number is out of range", () => {
		const text = "Line1\nLine2";
		expect(() => lineAndColumnToPosition(text, 3, 0)).toThrow();
	});

	it("should throw an error if the column number is out of range", () => {
		const text = "Line1\nLine2";
		expect(() => lineAndColumnToPosition(text, 1, 10)).toThrow();
	});
});
