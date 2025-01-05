import { bytesToBase64 } from "byte-base64";

export function serialize(data: Uint8Array): string {
	return bytesToBase64(data);
}
