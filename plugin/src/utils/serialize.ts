export function serialize(data: Uint8Array): string {
	return Buffer.from(data).toString("base64");
}
