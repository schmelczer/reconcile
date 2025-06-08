import type { Database } from "../persistence/database";
import type { Logger } from "../tracing/logger";
import type { Settings, SyncSettings } from "../persistence/settings";
import type { WebSocketServerMessage } from "./types/WebSocketServerMessage";
import type { Syncer } from "../sync-operations/syncer";
import type { WebSocketClientMessage } from "./types/WebSocketClientMessage";
import type { CursorPositionFromClient } from "./types/CursorPositionFromClient";
import type { ClientCursors } from "./types/ClientCursors";

export class WebSocketManager {
	private readonly webSocketStatusChangeListeners: (() => unknown)[] = [];
	private readonly remoteCursorsUpdateListeners: ((
		cursors: ClientCursors[]
	) => unknown)[] = [];

	private refreshWebSocketInterval: NodeJS.Timeout | undefined;

	private webSocket: WebSocket | undefined;

	private readonly webSocketFactoryImplementation: typeof globalThis.WebSocket;

	public constructor(
		private readonly deviceId: string,
		private readonly logger: Logger,
		private readonly database: Database,
		private readonly settings: Settings,
		private readonly syncer: Syncer,
		webSocketImplementation?: typeof globalThis.WebSocket
	) {
		if (webSocketImplementation) {
			this.webSocketFactoryImplementation = webSocketImplementation;
		} else {
			if (
				typeof globalThis !== "undefined" &&
				typeof globalThis.WebSocket === "undefined"
			) {
				// eslint-disable-next-line
				this.webSocketFactoryImplementation = require("ws"); // polyfill for WebSocket in Node.js
			} else {
				this.webSocketFactoryImplementation = WebSocket;
			}
		}

		this.updateWebSocket(settings.getSettings());

		settings.addOnSettingsChangeListener((newSettings, oldSettings) => {
			if (
				newSettings.remoteUri !== oldSettings.remoteUri ||
				newSettings.vaultName !== oldSettings.vaultName ||
				newSettings.token !== oldSettings.token ||
				newSettings.isSyncEnabled !== oldSettings.isSyncEnabled
			) {
				this.updateWebSocket(newSettings);
			}
		});

		this.setWebSocketRefreshInterval();
	}

	public get isWebSocketConnected(): boolean {
		return (
			this.webSocket?.readyState ===
			this.webSocketFactoryImplementation.OPEN
		);
	}

	public addWebSocketStatusChangeListener(listener: () => void): void {
		this.webSocketStatusChangeListeners.push(listener);
	}

	public addRemoteCursorsUpdateListener(
		listener: (cursors: ClientCursors[]) => void
	): void {
		this.remoteCursorsUpdateListeners.push(listener);
	}

	public async reset(): Promise<void> {
		this.setWebSocketRefreshInterval();
		this.updateWebSocket(this.settings.getSettings());
	}

	public stop(): void {
		clearInterval(this.refreshWebSocketInterval);

		try {
			this.webSocket?.close();
		} catch (e) {
			this.logger.warn(`Failed to close WebSocket: ${e}`);
		}
	}

	public updateLocalCursors(cursorPositions: CursorPositionFromClient): void {
		if (!this.isWebSocketConnected) {
			this.logger.warn(
				"WebSocket is not connected, cannot send cursor positions"
			);
			return;
		}
		const message: WebSocketClientMessage = {
			type: "cursorPositions",
			...cursorPositions
		};
		this.webSocket?.send(JSON.stringify(message));
		this.logger.info(
			`Sent cursor positions: ${JSON.stringify(cursorPositions)}`
		);
	}

	private updateWebSocket(settings: SyncSettings): void {
		try {
			this.webSocket?.close();
		} catch (e) {
			this.logger.warn(`Failed to close WebSocket: ${e}`);
		}

		if (!settings.isSyncEnabled) {
			this.webSocket = undefined;
			return;
		}

		const wsUri = new URL(settings.remoteUri);
		wsUri.protocol = wsUri.protocol === "https" ? "wss" : "ws";
		wsUri.pathname = `/vaults/${settings.vaultName}/ws`;

		this.logger.info(`Connecting to WebSocket at ${wsUri.toString()}`);

		this.webSocket = new this.webSocketFactoryImplementation(wsUri);

		this.webSocket.onmessage = async (event): Promise<void> => {
			// eslint-disable-next-line @typescript-eslint/no-unsafe-type-assertion
			const message = JSON.parse(event.data) as WebSocketServerMessage;

			if (message.type === "vaultUpdate") {
				try {
					await Promise.all(
						message.documents.map(async (document) =>
							this.syncer.syncRemotelyUpdatedFile(document)
						)
					);

					if (message.isInitialSync && message.documents.length > 0) {
						this.database.setLastSeenUpdateId(
							message.documents
								.map((document) => document.vaultUpdateId)
								.reduce((a, b) => Math.max(a, b))
						);
					}
				} catch (e) {
					this.logger.error(
						`Failed to sync remotely updated file: ${e}`
					);
				}
				// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
			} else if (message.type === "cursorPositions") {
				this.logger.info(
					`Received cursor positions for ${JSON.stringify(message.clients)}`
				);
				this.remoteCursorsUpdateListeners.forEach((listener) => {
					listener(
						message.clients.filter(
							(client) => client.deviceId !== this.deviceId
						)
					);
				});
			} else {
				this.logger.warn(
					`Received unknown message type: ${JSON.stringify(message)}`
				);
			}
		};

		// The JS WebSocket API doesn't support setting headers, so we have to send the token as a message
		this.webSocket.onopen = (): void => {
			this.logger.info("WebSocket connection opened");
			this.webSocketStatusChangeListeners.forEach((listener) => {
				listener();
			});

			const message: WebSocketClientMessage = {
				type: "handshake",
				deviceId: this.deviceId,
				token: settings.token,
				lastSeenVaultUpdateId: this.database.getLastSeenUpdateId()
			};
			this.webSocket?.send(JSON.stringify(message));
		};

		this.webSocket.onclose = (event): void => {
			this.logger.warn(
				`WebSocket closed with code ${event.code} (${event.reason == "" ? "unknown reason" : event.reason})`
			);
			this.webSocketStatusChangeListeners.forEach((listener) => {
				listener();
			});
		};
	}

	private setWebSocketRefreshInterval(): void {
		this.refreshWebSocketInterval = setInterval(() => {
			if (
				this.webSocket?.readyState ===
				this.webSocketFactoryImplementation.CLOSED
			) {
				this.logger.info("WebSocket is closed, reconnecting...");
				this.updateWebSocket(this.settings.getSettings());
			}
		}, this.settings.getSettings().webSocketRetryIntervalMs);
	}
}
