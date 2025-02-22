export function choose<T>(values: T[]): T {
	return values[Math.floor(Math.random() * values.length)];
}
