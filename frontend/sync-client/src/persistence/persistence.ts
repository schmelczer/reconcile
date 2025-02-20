export interface PersistenceProvider {
	load: () => Promise<unknown>;
	save: (data: unknown) => Promise<void>;
}
