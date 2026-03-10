/**
 * Test utility for detecting WASM memory leaks.
 *
 * wasm-bindgen registers every JS-side object with a `FinalizationRegistry`.
 * This detector patches `FinalizationRegistry.prototype.register` to collect
 * references to all WASM objects. After each test, {@link checkForWasmLeaks}
 * inspects `__wbg_ptr` on every tracked object - a non-zero pointer means
 * `.free()` was never called, i.e. a leak.
 *
 * Install once (before any WASM calls) and call {@link checkForWasmLeaks}
 * in an `afterEach` hook.
 */

let trackedObjects: object[] = [];
let originalRegister: Function | null = null;

interface WasmBindgenObject {
  __wbg_ptr: number;
  constructor: { name?: string };
}

function isWasmBindgenObject(target: unknown): target is WasmBindgenObject {
  return (
    target !== null &&
    typeof target === 'object' &&
    '__wbg_ptr' in (target as Record<string, unknown>)
  );
}

/**
 * Patches `FinalizationRegistry.prototype.register` to track all wasm-bindgen
 * objects. Safe to call multiple times (idempotent).
 */
export function installWasmLeakDetector(): void {
  if (originalRegister) return;

  originalRegister = FinalizationRegistry.prototype.register;

  FinalizationRegistry.prototype.register = function (
    target: object,
    heldValue: unknown,
    unregisterToken?: object
  ) {
    if (isWasmBindgenObject(target)) {
      trackedObjects.push(target);
    }
    return originalRegister!.call(this, target, heldValue, unregisterToken);
  };
}

/**
 * Returns any tracked WASM objects whose `__wbg_ptr` is still non-zero
 * (i.e. `.free()` was never called). Clears the tracked set afterwards.
 */
export function checkForWasmLeaks(): string[] {
  const leaks = trackedObjects
    .filter(isWasmBindgenObject)
    .filter((obj) => obj.__wbg_ptr !== 0)
    .map((obj) => `${obj.constructor?.name ?? 'Unknown'} (ptr=${obj.__wbg_ptr})`);

  trackedObjects = [];
  return leaks;
}
