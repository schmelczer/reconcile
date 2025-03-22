export interface PersistenceProvider<T> {
	load: () => Promise<T | undefined>;
	save: (data: T) => Promise<void>;
}
