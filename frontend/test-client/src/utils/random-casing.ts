export function randomCasing(str: string): string {
	const chars = str.split("");
	const randomCasedChars = chars.map((char) => {
		if (Math.random() < 0.5) {
			return char.toUpperCase();
		}
		return char.toLowerCase();
	});
	return randomCasedChars.join("");
}
