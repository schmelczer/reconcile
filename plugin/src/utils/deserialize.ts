import { base64ToBytes } from "byte-base64";

export function deserialize(data: string): Uint8Array {
	return base64ToBytes(data);
}
