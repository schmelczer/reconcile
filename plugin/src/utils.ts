// https://stackoverflow.com/questions/7616461/generate-a-hash-from-string-in-javascript
export function hash(content: Uint8Array): string {
	let hash = 0;
	for (let i = 0; i < content.length; i++) {
		hash = (hash << 5) - hash + content[i];
		hash |= 0; // convert to 32bit integer
	}
	return hash.toString(64);
}
