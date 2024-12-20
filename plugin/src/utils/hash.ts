// https://stackoverflow.com/questions/7616461/generate-a-hash-from-string-in-javascript
export function hash(content: Uint8Array): string {
	let result = 0;
	// eslint-disable-next-line @typescript-eslint/prefer-for-of
	for (let i = 0; i < content.length; i++) {
		result = (result << 5) - result + content[i];
		result |= 0; // Convert to 32bit integer
	}
	return Math.abs(result).toString(16);
}

export const EMPTY_HASH = hash(new Uint8Array(0));
