export function getRandomColor(name: string): string {
	let hash = 0;
	for (let i = 0; i < name.length; i++) {
		hash = (hash << 5) - hash + name.charCodeAt(i);
		hash |= 0; // Convert to 32bit integer
	}
	const normalised = hash / 0x7fffffff;
	return `hsl(${Math.abs(normalised * 360)}, 55%, 55%)`; // HSL color
}
