// Generates `pkg-rn/`: a React Native / Hermes-compatible build of the
// wasm-bindgen bindings in which the WebAssembly module is replaced by its
// wasm2js (pure-JS) translation.

import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { homedir } from 'node:os';

const here = dirname(fileURLToPath(import.meta.url));
const reconcileJsDir = resolve(here, '..');
const repoRoot = resolve(reconcileJsDir, '..');

const releaseWasm = resolve(
  repoRoot,
  'target/wasm32-unknown-unknown/release/reconcile_text.wasm'
);
const outDir = resolve(reconcileJsDir, 'pkg-rn');
const bgWasm = resolve(outDir, 'reconcile_text_bg.wasm');
const bgWasmJs = resolve(outDir, 'reconcile_text_bg.wasm.js');
const loweredWasm = resolve(outDir, '_lowered.wasm');
const entryJs = resolve(outDir, 'reconcile_text.js');

const wasmOpt = resolve(reconcileJsDir, 'node_modules/.bin/wasm-opt');
const wasm2js = resolve(reconcileJsDir, 'node_modules/.bin/wasm2js');

function run(cmd, args) {
  execFileSync(cmd, args, { stdio: 'inherit' });
}

// Locate the wasm-bindgen CLI. It MUST match the `wasm-bindgen` crate version pinned
// in Cargo.toml: a mismatched CLI emits bindings the runtime can't use. So we resolve
// the required version first and verify every candidate against it, failing loudly
// rather than silently falling back to whatever other version happens to be around.
function findWasmBindgen() {
  const cargoToml = readFileSync(resolve(repoRoot, 'Cargo.toml'), 'utf8');
  const wanted = cargoToml.match(
    /wasm-bindgen\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"/
  )?.[1];
  if (!wanted) {
    throw new Error(
      '[build-rn] Could not parse the pinned wasm-bindgen version from Cargo.toml, so ' +
        'the required CLI version is unknown. Has the dependency declaration changed?'
    );
  }

  // 1. On PATH: accept it only if its version matches the pin.
  let onPath = null;
  try {
    onPath = execFileSync('which', ['wasm-bindgen'], { encoding: 'utf8' }).trim();
  } catch {
    /* not on PATH; try the wasm-pack cache next */
  }
  if (onPath) {
    const version = execFileSync(onPath, ['--version'], { encoding: 'utf8' }).match(
      /\d+\.\d+\.\d+/
    )?.[0];
    if (version !== wanted) {
      throw new Error(
        `[build-rn] wasm-bindgen on PATH (${onPath}) is ${version ?? 'an unknown version'}, ` +
          `but Cargo.toml pins ${wanted}. Install the matching CLI ` +
          `(\`cargo install wasm-bindgen-cli --version ${wanted}\`) or remove the mismatched one.`
      );
    }
    return onPath;
  }

  const cacheRoots = [
    resolve(homedir(), 'Library/Caches/.wasm-pack'),
    resolve(homedir(), '.cache/.wasm-pack'),
  ];
  for (const root of cacheRoots) {
    if (!existsSync(root)) {
      continue;
    }
    for (const entry of readdirSync(root)) {
      const candidate = resolve(root, entry, 'wasm-bindgen');
      if (!existsSync(candidate)) {
        continue;
      }
      let version;
      try {
        version = execFileSync(candidate, ['--version'], { encoding: 'utf8' }).match(
          /\d+\.\d+\.\d+/
        )?.[0];
      } catch {
        continue; // not an invokable wasm-bindgen; ignore
      }
      if (version === wanted) {
        return candidate;
      }
    }
  }

  throw new Error(
    `[build-rn] No wasm-bindgen ${wanted} found on PATH or in the wasm-pack cache. ` +
      'Run `wasm-pack build --target web --features wasm` first (it caches the matching ' +
      `wasm-bindgen), or \`cargo install wasm-bindgen-cli --version ${wanted}\`.`
  );
}

if (!existsSync(releaseWasm)) {
  throw new Error(
    `Missing ${releaseWasm}.\nRun \`wasm-pack build --target web --features wasm\` from the repo root first.`
  );
}

console.log('[build-rn] generating bundler-target bindings with wasm-bindgen');
rmSync(outDir, { recursive: true, force: true });
const wasmBindgen = findWasmBindgen();
run(wasmBindgen, ['--target', 'bundler', '--out-dir', outDir, releaseWasm]);

// --- Patch wasm-bindgen's cached-memory getters for wasm2js -----------------
//
// wasm-bindgen caches typed-array / DataView views over `wasm.memory.buffer` and
// only re-creates them when it detects the heap grew. It detects a grow by looking
// for ArrayBuffer *detachment*: a real `WebAssembly.Memory.grow()` detaches the old
// buffer (its `byteLength` becomes 0 and `.detached` becomes true), and those are the
// only signals the generated getters check:
//   - getUint8ArrayMemory0(): refreshes when `byteLength === 0`           (detach only)
//   - getDataViewMemory0():   refreshes when `.detached === true`, OR when the buffer
//     identity changed but only `if (.detached === undefined)` — i.e. that identity
//     fallback runs solely on engines lacking `ArrayBuffer.prototype.detached`.
//
// wasm2js grows differently: `__wasm_memory_grow` (in reconcile_text_bg.wasm.js)
// allocates a NEW ArrayBuffer, copies the old heap into it, and reassigns
// `memory.buffer` WITHOUT ever detaching the old buffer. So the old buffer keeps
// `byteLength > 0` and `.detached === false`, and on modern engines that DO expose
// `ArrayBuffer.prototype.detached` (Node 25+, current Hermes) the identity fallback is
// gated off. Net effect: after a grow the getters keep returning views over the stale
// pre-grow buffer, silently corrupting any operation large enough to grow the heap.
// Small inputs never grow, so this escapes naive testing.
//
// WHY WE PATCH INSTEAD OF CONFIGURING.
// This is not fixed or configurable upstream: wasm-bindgen has no wasm2js / asm.js /
// React Native / "no-WebAssembly" target (every target assumes real WebAssembly
// detach-on-grow semantics), there is no flag to force buffer-identity comparison, and
// the getter-generation logic (crates/cli-support/src/js/mod.rs `memview`) is
// byte-for-byte identical from the pinned 0.2.114 through the latest release and
// `main`. The non-detaching-grow case is not even a tracked upstream issue. Rewriting
// the generated glue is therefore the only available fix: the two replacements below
// make BOTH getters also refresh on a buffer-identity change
// (`buffer !== wasm.memory.buffer`), which is the one signal wasm2js does give.
//
// Each replacement is asserted independently. If a future wasm-bindgen reshapes one
// getter but not the other, we MUST fail the build rather than ship a half-patched
// module whose un-patched getter corrupts large inputs. The post-build self-test at
// the bottom of this file is the backstop that proves the result survives a real grow.
const bgJsPath = resolve(outDir, 'reconcile_text_bg.js');
let bgJs = readFileSync(bgJsPath, 'utf8');

// (1) Uint8Array getter: append an unconditional buffer-identity check to the
//     `byteLength === 0` detach guard (upstream has no identity check here at all).
const byteLengthGuard = /(cached\w*Memory0)\.byteLength === 0/g;
const byteLengthHits = bgJs.match(byteLengthGuard)?.length ?? 0;
if (byteLengthHits === 0) {
  throw new Error(
    `[build-rn] Could not find the Uint8Array \`byteLength === 0\` growth guard in ` +
      `${bgJsPath} to patch for wasm2js. The wasm-bindgen output shape changed; update ` +
      'this patch (see crates/cli-support/src/js/mod.rs `memview`) — do NOT ship an ' +
      'unpatched getter, it will corrupt large inputs under wasm2js.'
  );
}
bgJs = bgJs.replace(
  byteLengthGuard,
  '$1.byteLength === 0 || $1.buffer !== wasm.memory.buffer'
);

// (2) DataView getter: drop the `detached === undefined &&` prefix so the existing
//     buffer-identity check runs on every runtime, not only legacy ones.
const gatedGuard =
  /(cached\w*Memory0)\.buffer\.detached === undefined && \1\.buffer !== wasm\.memory\.buffer/g;
const gatedHits = bgJs.match(gatedGuard)?.length ?? 0;
if (gatedHits === 0) {
  throw new Error(
    `[build-rn] Could not find the DataView \`detached === undefined\`-gated buffer-identity ` +
      `check in ${bgJsPath} to un-gate for wasm2js. The wasm-bindgen output shape changed; ` +
      'update this patch (see crates/cli-support/src/js/mod.rs `memview`) — do NOT ship an ' +
      'unpatched getter, it will corrupt large inputs under wasm2js.'
  );
}
bgJs = bgJs.replace(gatedGuard, '$1.buffer !== wasm.memory.buffer');

writeFileSync(bgJsPath, bgJs);

// Post-MVP features that wasm2js cannot translate must be lowered to MVP first.
// reference-types stays enabled: it only covers the funcref table here, which
// wasm2js handles via call_indirect.
const featureFlags = [
  '--enable-bulk-memory',
  '--enable-sign-ext',
  '--enable-nontrapping-float-to-int',
  '--enable-mutable-globals',
  '--enable-reference-types',
];

console.log('[build-rn] optimising and lowering to MVP with wasm-opt');
run(wasmOpt, [
  ...featureFlags,
  '-O3',
  '--signext-lowering',
  '--llvm-memory-copy-fill-lowering',
  '--llvm-nontrapping-fptoint-lowering',
  bgWasm,
  '-o',
  loweredWasm,
]);

console.log('[build-rn] translating wasm -> JS with wasm2js');
run(wasm2js, ['--enable-reference-types', loweredWasm, '-o', bgWasmJs]);

console.log('[build-rn] wiring the JS translation into reconcile_text.js');
const entry = readFileSync(entryJs, 'utf8');
const rewired = entry.replace(
  /from\s+(['"])\.\/reconcile_text_bg\.wasm\1/,
  'from $1./reconcile_text_bg.wasm.js$1'
);
if (rewired === entry) {
  throw new Error(
    `Could not find the \`./reconcile_text_bg.wasm\` import in ${entryJs}; ` +
      'the wasm-bindgen bundler output layout may have changed.'
  );
}
writeFileSync(entryJs, rewired);

// The binary and the intermediate are no longer referenced; remove them so no
// bundler attempts to instantiate WebAssembly from this directory.
rmSync(bgWasm, { force: true });
rmSync(loweredWasm, { force: true });

// Mark the directory as ESM (matching the web `pkg/`) so Node and Jest treat
// these `.js` files as modules. `sideEffects` stays true because importing the
// entry runs `__wbg_set_wasm(...)`, which must not be tree-shaken away.
writeFileSync(
  resolve(outDir, 'package.json'),
  JSON.stringify({ type: 'module', sideEffects: true }, null, 2) + '\n'
);

// Backstop: import the freshly generated module and prove it survives a heap grow.
// The patches above are matched by regex against wasm-bindgen output; a silently
// mis-applied patch (or a wasm-bindgen change we matched too loosely) would leave a
// getter reading the stale pre-grow buffer and corrupt large inputs only. Rather than
// trust the regexes, we force a grow here and assert a byte-exact round-trip, so a
// broken bundle fails the build instead of reaching a React Native consumer.
async function selfTest() {
  // Importing the entry runs `__wbg_set_wasm(...)`, initialising the wasm2js module.
  const api = await import(pathToFileURL(entryJs).href);
  // Same module instance (Node caches by resolved path), so this `memory` is the heap
  // the API operates on; its `.buffer` getter reflects the current (post-grow) buffer.
  const { memory } = await import(pathToFileURL(bgWasmJs).href);

  // ~100 KB of distinct tokens. The diff working set amplifies the input many-fold
  // (a ~50 KB input already forces dozens of grows), so this reliably grows the heap
  // well past wasm2js's ~1 MB initial allocation while staying fast. A tiny parent
  // keeps the edit distance — and therefore the runtime — small.
  const tokens = [];
  for (let i = 0; i < 10000; i++) {
    tokens.push(`token-${i}`);
  }
  const target = tokens.join(' ');
  const parent = 'reconcile self-test';

  const heapBefore = memory.buffer.byteLength;

  // Stale post-grow reads surface either as an out-of-bounds throw or as silently
  // wrong bytes, so handle both: a throw here is itself the failure signal.
  let roundTripped;
  try {
    const changed = new api.TextWithCursors(target, []);
    const compact = api
      .diff(parent, changed, 'Word')
      // This build's `undiff` rejects BigInt; normalise exactly as src/core.ts does.
      .map((item) => (typeof item === 'bigint' ? Number(item) : item));
    changed.free();
    roundTripped = api.undiff(parent, compact, 'Word');
  } catch (cause) {
    throw new Error(
      '[build-rn] self-test crashed during a large diff/undiff round-trip (after the heap ' +
        'grew). This is the signature of unpatched wasm2js cached-memory getters reading the ' +
        'stale pre-grow buffer. The growth patch is not taking effect. Refusing to ship this ' +
        'React Native bundle.',
      { cause }
    );
  }

  const heapAfter = memory.buffer.byteLength;

  if (heapAfter <= heapBefore) {
    throw new Error(
      `[build-rn] self-test did not grow the wasm heap (stayed at ${heapBefore} bytes), ` +
        'so it cannot validate the memory-growth patch. Enlarge the self-test input.'
    );
  }
  if (roundTripped !== target) {
    throw new Error(
      '[build-rn] self-test FAILED: diff/undiff round-trip did not match after a heap grow. ' +
        'The patched wasm2js cached-memory getters are returning stale/corrupt data — the ' +
        'growth patch is not taking effect. Refusing to ship this React Native bundle.'
    );
  }
}

console.log('[build-rn] self-testing the patched module (forces a heap grow)');
await selfTest();

console.log('[build-rn] done -> pkg-rn/');
