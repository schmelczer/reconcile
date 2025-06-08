import type {
	DocumentId,
	RelativePath,
	VaultUpdateId
} from "../persistence/database";

import type { Logger } from "../tracing/logger";
import type { Settings } from "../persistence/settings";
import type { ConnectionStatus } from "./connection-status";
import { sleep } from "../utils/sleep";
import { SyncResetError } from "./sync-reset-error";
import type { SerializedError } from "./types/SerializedError";
import type { DocumentVersionWithoutContent } from "./types/DocumentVersionWithoutContent";
import type { DocumentUpdateResponse } from "./types/DocumentUpdateResponse";
import type { DocumentVersion } from "./types/DocumentVersion";
import type { FetchLatestDocumentsResponse } from "./types/FetchLatestDocumentsResponse";
import type { PingResponse } from "./types/PingResponse";
import type { DeleteDocumentVersion } from "./types/DeleteDocumentVersion";

export interface CheckConnectionResult {
	isSuccessful: boolean;
	message: string;
}

export class SyncService {
	private static readonly NETWORK_RETRY_INTERVAL_MS = 1000;
	private readonly client: typeof globalThis.fetch;
	private readonly pingClient: typeof globalThis.fetch;

	public constructor(
		private readonly deviceId: string,
		private readonly connectionStatus: ConnectionStatus,
		private readonly settings: Settings,
		private readonly logger: Logger,
		fetchImplementation: typeof globalThis.fetch = globalThis.fetch
	) {
		// ensure that if it's called a method, `this` won't be bound to the instance
		const unboundFetch: typeof globalThis.fetch = async (...args) =>
			fetchImplementation(...args);

		this.client = this.connectionStatus.getFetchImplementation(
			this.logger,
			unboundFetch
		);
		this.pingClient = unboundFetch;
	}

	private static formatError(error: SerializedError): string {
		let result = error.message;
		if (error.causes.length > 0) {
			const causes = error.causes.join(", ");
			result += ` caused by: ${causes}`;
		}

		return result;
	}

	public async create({
		documentId,
		relativePath,
		contentBytes
	}: {
		documentId?: DocumentId;
		relativePath: RelativePath;
		contentBytes: Uint8Array;
	}): Promise<DocumentVersionWithoutContent> {
		return this.withRetries(async () => {
			const formData = new FormData();
			if (documentId !== undefined) {
				formData.append("document_id", documentId);
			}
			formData.append("relative_path", relativePath);
			formData.append("content", new Blob([contentBytes]));

			const response = await this.client(this.getUrl("/documents"), {
				method: "POST",
				body: formData,
				headers: this.getDefaultHeaders()
			});

			const result: SerializedError | DocumentVersionWithoutContent =
				(await response.json()) as  // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
					| SerializedError
					| DocumentVersionWithoutContent;

			if ("errorType" in result) {
				throw new Error(
					`Failed to create document: ${SyncService.formatError(result)}`
				);
			}

			this.logger.debug(
				`Created document ${JSON.stringify(result)} with id ${
					result.documentId
				}`
			);

			return result;
		});
	}

	public async put({
		parentVersionId,
		documentId,
		relativePath,
		contentBytes
	}: {
		parentVersionId: VaultUpdateId;
		documentId: DocumentId;
		relativePath: RelativePath;
		contentBytes: Uint8Array;
	}): Promise<DocumentUpdateResponse> {
		return this.withRetries(async () => {
			this.logger.debug(
				`Updating document ${documentId} with parent version ${parentVersionId} and relative path ${relativePath}`
			);
			const formData = new FormData();
			formData.append("parent_version_id", parentVersionId.toString());
			formData.append("relative_path", relativePath);
			formData.append("content", new Blob([contentBytes]));

			const response = await this.client(
				this.getUrl(`/documents/${documentId}`),
				{
					method: "PUT",
					body: formData,
					headers: this.getDefaultHeaders()
				}
			);

			const result: SerializedError | DocumentUpdateResponse =
				(await response.json()) as  // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
					| SerializedError
					| DocumentUpdateResponse;

			if ("errorType" in result) {
				throw new Error(
					`Failed to update document: ${SyncService.formatError(result)}`
				);
			}

			this.logger.debug(
				`Updated document ${JSON.stringify(result)} with id ${
					result.documentId
				}}`
			);

			return result;
		});
	}

	public async delete({
		documentId,
		relativePath
	}: {
		documentId: DocumentId;
		relativePath: RelativePath;
	}): Promise<DocumentVersionWithoutContent> {
		return this.withRetries(async () => {
			const request: DeleteDocumentVersion = {
				relativePath
			};
			const response = await this.client(
				this.getUrl(`/documents/${documentId}`),
				{
					method: "DELETE",
					body: JSON.stringify(request),
					headers: {
						"Content-Type": "application/json",
						...this.getDefaultHeaders()
					}
				}
			);

			const result: SerializedError | DocumentVersionWithoutContent =
				(await response.json()) as  // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
					| SerializedError
					| DocumentVersionWithoutContent;

			if ("errorType" in result) {
				throw new Error(
					`Failed to delete document: ${SyncService.formatError(result)}`
				);
			}

			this.logger.debug(
				`Deleted document ${relativePath} with id ${documentId}`
			);

			return result;
		});
	}

	public async get({
		documentId
	}: {
		documentId: DocumentId;
	}): Promise<DocumentVersion> {
		return this.withRetries(async () => {
			const response = await this.client(
				this.getUrl(`/documents/${documentId}`),
				{
					headers: this.getDefaultHeaders()
				}
			);

			const result: SerializedError | DocumentVersion =
				(await response.json()) as SerializedError | DocumentVersion; // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion

			if ("errorType" in result) {
				throw new Error(
					`Failed to get document: ${SyncService.formatError(result)}`
				);
			}

			this.logger.debug(
				`Get document ${result.relativePath} with id ${result.documentId}`
			);

			return result;
		});
	}

	public async getAll(
		since?: VaultUpdateId
	): Promise<FetchLatestDocumentsResponse> {
		return this.withRetries(async () => {
			const url = new URL(this.getUrl("/documents"));
			if (since !== undefined) {
				url.searchParams.append("since", since.toString());
			}
			const response = await this.client(url.toString(), {
				headers: this.getDefaultHeaders()
			});

			const result: SerializedError | FetchLatestDocumentsResponse =
				(await response.json()) as  // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion
					| SerializedError
					| FetchLatestDocumentsResponse;

			if ("errorType" in result) {
				throw new Error(
					`Failed to get documents: ${SyncService.formatError(result)}`
				);
			}

			this.logger.debug(
				`Got ${result.latestDocuments.length} document metadata`
			);

			return result;
		});
	}

	public async checkConnection(): Promise<CheckConnectionResult> {
		try {
			const response = await this.pingClient(this.getUrl("/ping"), {
				headers: this.getDefaultHeaders()
			});
			const result: PingResponse | SerializedError =
				(await response.json()) as PingResponse | SerializedError; // eslint-disable-line @typescript-eslint/no-unsafe-type-assertion

			if ("errorType" in result) {
				throw new Error(
					`Failed to ping server: ${SyncService.formatError(result)}`
				);
			}

			if (result.isAuthenticated) {
				return {
					isSuccessful: true,
					message: `Successfully connected to server (version: ${result.serverVersion}) and authenticated`
				};
			}

			return {
				isSuccessful: false,
				message: `Successfully connected to server (version: ${result.serverVersion}) but failed to authenticate`
			};
		} catch (e) {
			return {
				isSuccessful: false,
				message: `Failed to connect to server: ${e}`
			};
		}
	}

	private getUrl(path: string): string {
		const { vaultName, remoteUri } = this.settings.getSettings();
		const safeRemoteUri = remoteUri.replace(/\/+$/, "");
		return `${safeRemoteUri}/vaults/${vaultName}${path}`;
	}

	private getDefaultHeaders(): Record<string, string> {
		return {
			"device-id": this.deviceId,
			authorization: `Bearer ${this.settings.getSettings().token}`
		};
	}

	private async withRetries<T>(fn: () => Promise<T>): Promise<T> {
		// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
		while (true) {
			try {
				return await fn();
			} catch (e) {
				// We must not retry errors coming from reset
				if (e instanceof SyncResetError) {
					throw e;
				}

				this.logger.error(
					`Failed network call (${e}), retrying in ${SyncService.NETWORK_RETRY_INTERVAL_MS}ms`
				);
				await sleep(SyncService.NETWORK_RETRY_INTERVAL_MS);
			}
		}
	}
}
