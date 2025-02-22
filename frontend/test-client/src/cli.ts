import { SyncSettings } from "sync-client";
import { MockAgent } from "./agent/mock-agent";
import { sleep } from "./utils/sleep";
import { v4 as uuidv4 } from "uuid";

const globalFiles: Record<string, Uint8Array> = {};
const iterations = 100;

async function runTest(): Promise<void> {
	console.info("Starting test...");

	const initialSettings: Partial<SyncSettings> = {
		isSyncEnabled: true,
		token: "token",
		vaultName: uuidv4()
	};

	const clients = [
		new MockAgent(globalFiles, initialSettings, "agent-1"),
		new MockAgent(globalFiles, initialSettings, "agent-2"),
		new MockAgent(globalFiles, initialSettings, "agent-3"),
		new MockAgent(globalFiles, initialSettings, "agent-4"),
		new MockAgent(globalFiles, initialSettings, "agent-5")
	];

	await Promise.all(clients.map((client) => client.init()));

	for (let i = 0; i < iterations; i++) {
		await Promise.all(clients.map((client) => client.act()));
		await sleep(100);
	}

	await Promise.all(clients.map((client) => client.finish()));

	clients.forEach((client) => {
		client.assertFileSystemIsConsistent();
		client.assertAllContentIsPresentOnce();
	});

	console.info("Test completed successfully");
}

runTest();
