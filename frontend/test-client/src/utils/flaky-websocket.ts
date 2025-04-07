import { sleep } from "./sleep";

export function flakyWebSocketFactory(
	jitterScaleInSeconds: number
): typeof WebSocket {
	// eslint-disable-next-line
	return class FlakyWebSocket extends require("ws") {
		public set onopen(callback: (event: Event) => void) {
			// eslint-disable-next-line
			super.onopen = async (event: Event): Promise<void> => {
				if (jitterScaleInSeconds > 0) {
					await sleep(Math.random() * jitterScaleInSeconds * 1000);
				}

				callback(event);
			};
		}

		public set onmessage(callback: (event: MessageEvent) => void) {
			// eslint-disable-next-line
			super.onmessage = async (event: MessageEvent): Promise<void> => {
				if (jitterScaleInSeconds > 0) {
					await sleep(Math.random() * jitterScaleInSeconds * 1000);
				}

				callback(event);
			};
		}

		public set onclose(callback: (event: CloseEvent) => void) {
			// eslint-disable-next-line
			super.onclose = async (event: CloseEvent): Promise<void> => {
				if (jitterScaleInSeconds > 0) {
					await sleep(Math.random() * jitterScaleInSeconds * 1000);
				}
				callback(event);
			};
		}

		public set onerror(callback: (event: Event) => void) {
			// eslint-disable-next-line
			super.onerror = async (event: Event): Promise<void> => {
				if (jitterScaleInSeconds > 0) {
					await sleep(Math.random() * jitterScaleInSeconds * 1000);
				}
				callback(event);
			};
		}

		public async send(
			data: string | ArrayBufferLike | Blob | ArrayBufferView
		): Promise<void> {
			if (jitterScaleInSeconds > 0) {
				await sleep(Math.random() * jitterScaleInSeconds * 1000);
			}

			// eslint-disable-next-line
			super.send(data);
		}
	} as unknown as typeof WebSocket;
}
