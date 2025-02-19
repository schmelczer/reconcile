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
	private static readonly MAX_MESSAGES = 1000;

	private static instance: Logger | null = null;
	private readonly messages: LogLine[] = [];

	private readonly onMessageListeners: ((
		status: LogLine | undefined
	) => void)[] = [];

	private constructor() {} // eslint-disable-line @typescript-eslint/no-empty-function

	public static getInstance(): Logger {
		if (!Logger.instance) {
			Logger.instance = new Logger();
		}
		return Logger.instance;
	}

	public debug(message: string): void {
		console.debug(message);
		this.pushMessage(message, LogLevel.DEBUG);
	}

	public info(message: string): void {
		console.info(message);
		this.pushMessage(message, LogLevel.INFO);
	}

	public warn(message: string): void {
		console.warn(message);
		this.pushMessage(message, LogLevel.WARNING);
	}

	public error(message: string): void {
		console.error(message);
		this.pushMessage(message, LogLevel.ERROR);
	}

	public getMessages(mininumSeverity: LogLevel): LogLine[] {
		return this.messages.filter(
			(message) =>
				LOG_LEVEL_ORDER[message.level] >=
				LOG_LEVEL_ORDER[mininumSeverity]
		);
	}

	public addOnMessageListener(
		listener: (message: LogLine | undefined) => void
	): void {
		this.onMessageListeners.push(listener);
	}

	public reset(): void {
		this.messages.length = 0;
		this.onMessageListeners.forEach((listener) => {
			listener(undefined);
		});
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
