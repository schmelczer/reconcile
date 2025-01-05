export function deserialize(data: string): Uint8Array {
	return Buffer.from(data, "base64");
}
