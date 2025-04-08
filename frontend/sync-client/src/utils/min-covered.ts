/**
 * A class that tracks the minimum covered value in a sequence of numbers.
 * It keeps track of a minimum value based on the seen values.
 *
 * It expects integers slightly out of order and makes sure that the value of `min` is
 * always the minimum of the seen values. This is done with bounded memory usage.
 *
 * @example
 * ```typescript
 * const covered = new CoveredValues(0);
 * covered.add(2); // seenValues = [2], min = 0
 * covered.add(1); // seenValues = [], min = 2
 * covered.min; // returns 2
 * ```
 */
export class CoveredValues {
	private seenValues: number[] = [];

	public constructor(private minValue: number) {}

	public add(value: number): void {
		if (value < this.minValue) {
			return;
		}

		let i = 0;
		while (i < this.seenValues.length && this.seenValues[i] < value) {
			i++;
		}

		if (i === this.seenValues.length) {
			this.seenValues.push(value);
		} else if (this.seenValues[i] === value) {
			return;
		} else {
			this.seenValues.splice(i, 0, value);
		}

		while (
			this.seenValues.length > 0 &&
			this.seenValues[0] === this.minValue + 1
		) {
			this.seenValues.shift();
			this.minValue++;
		}
	}

	public get min(): number {
		return this.minValue;
	}
}
