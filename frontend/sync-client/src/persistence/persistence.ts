export interface PersistenceProvider<T extends object> {
	load: () => Promise<T | undefined>;
	save: (data: T | undefined) => Promise<void>;
}
