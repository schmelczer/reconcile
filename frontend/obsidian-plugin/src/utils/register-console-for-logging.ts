import type { LogLine, SyncClient } from "sync-client";
import { LogLevel } from "sync-client";

export function registerConsoleForLogging(client: SyncClient): void {
	client.logger.addOnMessageListener((logLine: LogLine) => {
		const formatted = `${logLine.timestamp.toISOString()} ${logLine.level} ${logLine.message}`;

		switch (logLine.level) {
			case LogLevel.ERROR:
				console.error(formatted);
				break;
			case LogLevel.WARNING:
				console.warn(formatted);
				break;
			case LogLevel.INFO:
				console.info(formatted);
				break;
			case LogLevel.DEBUG:
				console.debug(formatted);
				break;
		}
	});
}
