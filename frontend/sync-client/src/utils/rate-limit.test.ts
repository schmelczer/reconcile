import { rateLimit } from "./rate-limit";
import { jest } from "@jest/globals";

describe("rateLimit", () => {
	beforeEach(() => {
		jest.useFakeTimers();
	});

	afterEach(() => {
		jest.useRealTimers();
	});

	it("should call the function immediately on first invocation", async () => {
		const mockFn = jest
			.fn<() => Promise<string>>()
			.mockResolvedValue("result");
		const rateLimited = rateLimit(mockFn, 100);

		const promise = rateLimited();
		expect(mockFn).toHaveBeenCalledTimes(1);

		await promise;
	});

	it("should call the function again after the interval has passed", async () => {
		const mockFn = jest
			.fn<(value: number) => Promise<string>>()
			.mockResolvedValue("result");

		const rateLimited = rateLimit(mockFn, 100);

		const promise1 = rateLimited(1);
		await promise1;

		jest.advanceTimersByTime(200);

		const promise2 = rateLimited(2);
		await promise2;

		expect(mockFn).toHaveBeenCalledTimes(2);
		expect(mockFn).toHaveBeenCalledWith(2);
	});

	it("should use the most recent arguments if multiple calls are made within interval", async () => {
		const mockFn = jest
			.fn<(value: string) => Promise<string>>()
			.mockImplementation(async (val) => `${val}-result`);
		const rateLimited = rateLimit(mockFn, 100);

		const promise1 = rateLimited("first");
		jest.advanceTimersByTime(10);
		const promise2 = rateLimited("second");
		jest.advanceTimersByTime(10);
		const promise3 = rateLimited("third");

		jest.advanceTimersByTime(1000);

		expect(await promise1).toEqual("first-result");
		expect(await promise2).toEqual("third-result");
		expect(await promise3).toBeUndefined();

		expect(mockFn).toHaveBeenCalledTimes(2);
		expect(mockFn).toHaveBeenNthCalledWith(1, "first");
		expect(mockFn).toHaveBeenNthCalledWith(2, "third");
	});
});
