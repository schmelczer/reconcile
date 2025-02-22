export enum LogLevel {
	DEBUG = "DEBUG",
	INFO = "INFO",
	WARNING = "WARNING",
	ERROR = "ERROR"
}

const LOG_LEVEL_ORDER = {
	[LogLevel.DEBUG]: 0,
	[LogLevel.INFO]: 1,
	[LogLevel.WARNING]: 2,
	[LogLevel.ERROR]: 3
};

export class LogLine {
	public timestamp = new Date();
	public constructor(
		public level: LogLevel,
		public message: string
	) {}
}

export class Logger {
	private static readonly MAX_MESSAGES = 10000;
	private readonly messages: LogLine[] = [];
	private readonly onMessageListeners: ((message: LogLine) => void)[] = [];

	public constructor(...onMessageListeners: ((message: LogLine) => void)[]) {
		this.onMessageListeners = onMessageListeners;
	}

	public debug(message: string): void {
		this.pushMessage(message, LogLevel.DEBUG);
	}

	public info(message: string): void {
		this.pushMessage(message, LogLevel.INFO);
	}

	public warn(message: string): void {
		this.pushMessage(message, LogLevel.WARNING);
	}

	public error(message: string): void {
		this.pushMessage(message, LogLevel.ERROR);
	}

	public getMessages(mininumSeverity: LogLevel): LogLine[] {
		return this.messages.filter(
			(message) =>
				LOG_LEVEL_ORDER[message.level] >=
				LOG_LEVEL_ORDER[mininumSeverity]
		);
	}

	public addOnMessageListener(listener: (message: LogLine) => void): void {
		this.onMessageListeners.push(listener);
	}

	public reset(): void {
		this.messages.length = 0;
		this.debug("Logger has been reset");
	}

	private pushMessage(message: string, level: LogLevel): void {
		const logLine = new LogLine(level, message);
		this.messages.push(logLine);

		while (this.messages.length > Logger.MAX_MESSAGES) {
			this.messages.shift();
		}

		this.onMessageListeners.forEach((listener) => {
			listener(logLine);
		});
	}
}
