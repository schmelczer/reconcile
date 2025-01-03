import { isEqualBytes } from "./is-equal-bytes";

describe("isEqualBytes", () => {
	it("should return true for equal byte arrays", () => {
		const bytes1 = new Uint8Array([1, 2, 3, 4]);
		const bytes2 = new Uint8Array([1, 2, 3, 4]);
		expect(isEqualBytes(bytes1, bytes2)).toBe(true);
	});

	it("should return false for byte arrays of different lengths", () => {
		const bytes1 = new Uint8Array([1, 2, 3, 4]);
		const bytes2 = new Uint8Array([1, 2, 3]);
		expect(isEqualBytes(bytes1, bytes2)).toBe(false);
	});

	it("should return true for empty byte arrays", () => {
		const bytes1 = new Uint8Array([]);
		const bytes2 = new Uint8Array([]);
		expect(isEqualBytes(bytes1, bytes2)).toBe(true);
	});

	it("should return false for byte arrays with same length but different content", () => {
		const bytes1 = new Uint8Array([1, 2, 3, 4]);
		const bytes2 = new Uint8Array([4, 3, 2, 1]);
		expect(isEqualBytes(bytes1, bytes2)).toBe(false);
	});
});
