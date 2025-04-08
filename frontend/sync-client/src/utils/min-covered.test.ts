import { CoveredValues } from "./min-covered";

describe("CoveredValues", () => {
	test("should initialize with the given min value", () => {
		const covered = new CoveredValues(5);
		expect(covered.min).toBe(5);
	});

	test("should add values greater than min", () => {
		const covered = new CoveredValues(0);
		covered.add(3);
		expect(covered.min).toBe(0);
		covered.add(1);
		expect(covered.min).toBe(1);
		covered.add(4);
		expect(covered.min).toBe(1);
		covered.add(2);
		expect(covered.min).toBe(4);
	});

	test("should ignore duplicate values", () => {
		const covered = new CoveredValues(0);
		covered.add(3);
		covered.add(3);
		covered.add(3);
		expect(covered.min).toBe(0);
		covered.add(1);
		covered.add(2);
		expect(covered.min).toBe(3);
	});

	test("should handle multiple consecutive values", () => {
		const covered = new CoveredValues(132);
		for (let i = 250; i > 132; i--) {
			expect(covered.min).toBe(132);
			covered.add(i);
		}
		expect(covered.min).toBe(250);
	});

	test("should handle adding values lower than current min", () => {
		const covered = new CoveredValues(5);
		covered.add(3);
		expect(covered.min).toBe(5);
		covered.add(6);
		expect(covered.min).toBe(6);
	});

	test("should handle force setting min value", () => {
		const covered = new CoveredValues(5);
		covered.add(7);
		covered.add(8);
		covered.add(9);
		expect(covered.min).toBe(5);
		covered.min = 6;
		expect(covered.min).toBe(6);
		covered.add(10);
		expect(covered.min).toBe(10);
	});
});
