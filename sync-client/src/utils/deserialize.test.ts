import init, { base64ToBytes } from "sync_lib";
import fs from "fs";

describe("deserialize", () => {
	it("should serialize a Uint8Array to a base64 string", async () => {
		const wasmBin = fs.readFileSync(
			"../backend/sync_lib/pkg/sync_lib_bg.wasm"
		);
		await init({ module_or_path: wasmBin });

		const base64 = "SGVsbG8=";
		const jsResult = base64ToBytes(base64);
		const expected = new Uint8Array([72, 101, 108, 108, 111]);
		expect(jsResult).toEqual(expected);
		const rustResult = base64ToBytes(base64);
		expect(jsResult).toEqual(rustResult);
	});
});
