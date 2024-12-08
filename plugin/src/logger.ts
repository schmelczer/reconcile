enum LogLevel {
	DEBUG,
	INFO,
	WARNING,
	ERROR,
}

class LogLine {
	constructor(public level: LogLevel, public message: string) {}

	public toString(): string {
		return `${this.formatLevel()}: ${this.message}`;
	}

	private formatLevel(): string {
		switch (this.level) {
			case LogLevel.DEBUG:
				return "DEBUG";
			case LogLevel.INFO:
				return "INFO";
			case LogLevel.WARNING:
				return "WARNING";
			case LogLevel.ERROR:
				return "ERROR";
			default:
				return "UNKNOWN";
		}
	}
}

export class Logger {
	private static readonly MAX_MESSAGES = 1000;

	private static instance: Logger;
	private messages: LogLine[] = [];

	private constructor() {}

	static getInstance(): Logger {
		if (!Logger.instance) {
			Logger.instance = new Logger();
		}
		return Logger.instance;
	}

	public debug(message: string): void {
		this.pushMessage(message, LogLevel.DEBUG);
		console.debug(message);
	}

	public info(message: string): void {
		this.pushMessage(message, LogLevel.INFO);
		console.log(message);
	}

	public warn(message: string): void {
		this.pushMessage(message, LogLevel.WARNING);
		console.warn(message);
	}

	public error(message: string): void {
		this.pushMessage(message, LogLevel.ERROR);
		console.error(message);
	}

	public getMessages(): LogLine[] {
		return this.messages;
	}

	private pushMessage(message: string, level: LogLevel): void {
		this.messages.push(new LogLine(level, message));
		if (this.messages.length > Logger.MAX_MESSAGES) {
			this.messages.shift();
		}
	}
}
