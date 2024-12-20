import { Notice } from "obsidian";

export enum LogLevel {
	DEBUG = "DEBUG",
	INFO = "INFO",
	WARNING = "WARNING",
	ERROR = "ERROR",
}

class LogLine {
	public timestamp = new Date();
	public constructor(public level: LogLevel, public message: string) {}

	public toString(): string {
		return `| ${this.formatLevel()} | ${this.timestamp.getHours()}:${this.timestamp.getMinutes()}:${this.timestamp.getSeconds()} | ${
			this.message
		}`;
	}

	private formatLevel(): string {
		switch (this.level) {
			case LogLevel.DEBUG:
				return "  DEBUG";
			case LogLevel.INFO:
				return "   INFO";
			case LogLevel.WARNING:
				return "WARNING";
			case LogLevel.ERROR:
				return "  ERROR";
			default:
				return "UNKNOWN";
		}
	}
}

export class Logger {
	private static readonly MAX_MESSAGES = 1000;

	private static instance: Logger | null = null;
	private readonly messages: LogLine[] = [];

	private constructor() {} // eslint-disable-line @typescript-eslint/no-empty-function

	public static getInstance(): Logger {
		if (!Logger.instance) {
			Logger.instance = new Logger();
		}
		return Logger.instance;
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
		new Notice(message, 5000);
	}

	public getMessages(mininumSeverity: LogLevel): LogLine[] {
		return this.messages.filter(
			(message) => message.level >= mininumSeverity
		);
	}

	private pushMessage(message: string, level: LogLevel): void {
		this.messages.push(new LogLine(level, message));
		if (this.messages.length > Logger.MAX_MESSAGES) {
			this.messages.shift();
		}
	}
}
