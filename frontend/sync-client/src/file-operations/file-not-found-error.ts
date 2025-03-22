export class FileNotFoundError extends Error {
	public constructor(message: string) {
		super(message);
		this.name = "FileNotFoundError";
	}
}
