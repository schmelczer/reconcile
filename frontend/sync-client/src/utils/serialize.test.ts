import { serialize } from "./serialize";
import init, { bytesToBase64 } from "sync_lib";
import fs from "fs";

describe("serialize", () => {
	it("should serialize a Uint8Array to a base64 string", async () => {
		const wasmBin = fs.readFileSync(
			"../../backend/sync_lib/pkg/sync_lib_bg.wasm"
		);
		await init({ module_or_path: wasmBin });

		const data = new Uint8Array([72, 101, 108, 108, 111]);
		const jsResult = serialize(data);
		const rustResult = bytesToBase64(data);
		expect(rustResult).toBe("SGVsbG8=");
		expect(jsResult).toBe(rustResult);
	});
});
