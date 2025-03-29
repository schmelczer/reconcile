import { createPromise } from "./create-promise";
import { sleep } from "./sleep";

/**
 * Creates a rate-limited version of a given asynchronous function.
 * Ensures that the function is not called more frequently than specified by `minIntervalMs`.
 * If the function is called while a previous call is still within the rate limit window,
 * it will queue up the most recent arguments and execute them after the rate limit expires.
 * Only the most recent call is preserved in the queue.
 *
 * @template T - Type of the function to be rate limited
 * @param {T} fn - The asynchronous function to rate limit
 * @param {number} minIntervalMs - The minimum interval in milliseconds between function calls
 * @returns {(...args: Parameters<T>) => ReturnType<T> | Promise<undefined>} A decorated function that respects the rate limit.
 *         Returns the original function's return type when executed, or undefined if the call was superseded by a newer one.
 */
export function rateLimit<
	R,
	T extends (
		...args: any // eslint-disable-line @typescript-eslint/no-explicit-any
	) => Promise<R>
>(
	fn: T,
	minIntervalMs: number
): (...args: Parameters<T>) => Promise<R | undefined> {
	let newArgs: Parameters<T> | undefined = undefined;
	let running: Promise<unknown> | undefined = undefined;

	const decoratedFn = async (
		...args: Parameters<T>
	): Promise<R | undefined> => {
		if (running !== undefined) {
			newArgs = args;
			await running;

			// args might have changed while we were waiting
			// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
			if (newArgs === undefined) {
				// we weren't the first one to wake up, that means a newer
				// invocation is running now, we can just bail
				return;
			}
			args = newArgs;
			newArgs = undefined;
		}

		const [promise, resolve] = createPromise();
		running = promise;
		sleep(minIntervalMs)
			.then(resolve)
			.catch(() => {
				// sleep cannot fail
			});
		return fn(...args);
	};

	return decoratedFn;
}
