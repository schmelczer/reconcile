import type { SyncSettings } from "sync-client";
import { MockAgent } from "./agent/mock-agent";
import { sleep } from "./utils/sleep";
import { v4 as uuidv4 } from "uuid";
import { randomCasing } from "./utils/random-casing";

let slowFileEvents = false;

async function runTest({
	agentCount,
	concurrency,
	iterations,
	doDeletes,
	useSlowFileEvents,
	jitterScaleInSeconds
}: {
	agentCount: number;
	concurrency: number;
	iterations: number;
	doDeletes: boolean;
	useSlowFileEvents: boolean;
	jitterScaleInSeconds: number;
}): Promise<void> {
	slowFileEvents = useSlowFileEvents;

	const settings = `with ${agentCount} agents, concurrency ${concurrency}, iterations ${iterations}, doDeletes ${doDeletes}, jitterScaleInSeconds ${jitterScaleInSeconds}, useSlowFileEvents ${useSlowFileEvents}`;
	console.info(`Running test ${settings}`);

	const vaultName = uuidv4();
	console.info(`Using vault name: ${vaultName}`);
	const initialSettings: Partial<SyncSettings> = {
		isSyncEnabled: true,
		token: "   test-token-change-me     ", // same as in backend/config-e2e.yml with spaces
		vaultName: randomCasing(vaultName) + (Math.random() > 0.5 ? "  " : ""), // extra spaces shouldn't matter
		syncConcurrency: concurrency,
		remoteUri: "http://localhost:3000"
	};

	const clients: MockAgent[] = [];
	for (let i = 0; i < agentCount; i++) {
		clients.push(
			new MockAgent(
				initialSettings,
				`agent-${i}`,
				doDeletes,
				useSlowFileEvents,
				jitterScaleInSeconds
			)
		);
	}

	try {
		await Promise.all(clients.map(async (client) => client.init()));

		for (let i = 0; i < iterations; i++) {
			console.info(`Iteration ${i + 1}/${iterations}`);
			await Promise.all(clients.map(async (client) => client.act()));
			await sleep(100);
		}

		console.info("Stopping agents");

		// Each agent can have unpushed changes which might conflict with eachother so each has to resolve the conflicts & push, and
		for (const client of clients) {
			try {
				await client.finish();
			} catch (err) {
				if (!slowFileEvents) {
					throw err;
				}
			}
		}

		// then we need a second pass to ensure that all agents pull the same state.
		for (const client of clients) {
			try {
				await client.finish();
			} catch (err) {
				if (!slowFileEvents) {
					throw err;
				}
			}
		}

		console.info("Agents finished successfully");

		clients.slice(0, -1).forEach((client, i) => {
			console.info(
				`Checking consistency between ${client.name} and ${clients[i + 1].name}`
			);
			client.assertFileSystemsAreConsistent(clients[i]);
			console.info(`Consistency check for ${client.name} passed`);
		});

		console.info("File systems found to be consistent");

		clients.forEach((client) => {
			console.info(`Checking content for ${client.name}`);
			client.assertAllContentIsPresentOnce();
			console.info(`Content check for ${client.name} passed`);
		});

		console.info(`Test passed ${settings}`);
	} catch (err) {
		console.error(`Test failed ${settings}`);
		throw err;
	}
}

async function runTests(): Promise<void> {
	for (const useSlowFileEvents of [false, true]) {
		for (const concurrency of [
			16,
			1 // test with concurrency 1 to check for deadlocks
		]) {
			for (const doDeletes of [true, false]) {
				await runTest({
					agentCount: 2,
					concurrency,
					iterations: 100,
					doDeletes,
					useSlowFileEvents,
					jitterScaleInSeconds: 0.75
				});
			}
		}
	}
}

process.on("uncaughtException", (error) => {
	if (slowFileEvents) {
		return;
	}
	console.error("Uncaught Exception:", error);
	process.exit(1);
});

process.on("unhandledRejection", (error, _promise) => {
	if (error instanceof Error && error.message === "Sync was reset") {
		return;
	}

	if (slowFileEvents) {
		return;
	}

	console.error("Unhandled Rejection:", error);
	process.exit(1);
});

runTests()
	.then(() => {
		process.exit(0);
	})
	.catch((err: unknown) => {
		console.error(err);
		process.exit(1);
	});
