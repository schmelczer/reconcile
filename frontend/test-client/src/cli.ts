import { SyncSettings } from "sync-client";
import { MockAgent } from "./agent/mock-agent";
import { sleep } from "./utils/sleep";
import { v4 as uuidv4 } from "uuid";

const globalFiles: Record<string, Uint8Array> = {};
const iterations = 100;
const doDeletes = false;

async function runTest(): Promise<void> {
	console.info("Starting test");

	const initialSettings: Partial<SyncSettings> = {
		isSyncEnabled: true,
		token: "token",
		vaultName: uuidv4(),
		remoteUri: "http://localhost:3030"
	};

	const clients = [
		new MockAgent(
			globalFiles,
			initialSettings,
			"agent-1",
			"#ff0000",
			doDeletes
		),
		new MockAgent(
			globalFiles,
			initialSettings,
			"agent-2",
			"#00ff00",
			doDeletes
		),
		new MockAgent(
			globalFiles,
			initialSettings,
			"agent-3",
			"#0000ff",
			doDeletes
		),
		new MockAgent(
			globalFiles,
			initialSettings,
			"agent-4",
			"#ffaa00",
			doDeletes
		),
		new MockAgent(
			globalFiles,
			initialSettings,
			"agent-5",
			"#00ffaa",
			doDeletes
		)
	];

	await Promise.all(clients.map((client) => client.init()));

	for (let i = 0; i < iterations; i++) {
		await Promise.all(clients.map((client) => client.act()));
		await sleep(100);
	}

	await Promise.all(clients.map((client) => client.finish()));

	console.info("Agents finished successfully");

	clients.forEach((client) => {
		console.info(`Checking consistency for ${client.name}`);
		client.assertFileSystemIsConsistent();
		console.info(`Consistency check for ${client.name} passed`);
	});

	console.info("File systems found to be consistent");

	clients.forEach((client) => {
		console.info(`Checking content for ${client.name}`);
		client.assertAllContentIsPresentOnce();
		console.info(`Content check for ${client.name} passed`);
	});

	console.info("Test completed successfully");
}

runTest();
