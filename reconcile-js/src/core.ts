// Shared, platform-agnostic wrapper around the generated wasm-bindgen surface.
//
// The actual wasm bindings are injected by a platform-specific entrypoint:
//   - `index.ts`     (web/node) instantiates the real WebAssembly module lazily
//                    on first use via `initSync`.
//   - `index.rn.ts`  (React Native / Hermes) links a wasm2js (pure-JS)
//                    implementation, since Hermes does not expose a runtime
//                    `WebAssembly` global. See `scripts/build-rn.mjs`.

// Keep these structural types local: the raw wasm-bindgen package and this
// wrapper share a name, so an import of 'reconcile-text' in emitted declarations
// would resolve to the wrapper's public API in a consumer.
interface WasmCursorPosition {
  id(): number;
  characterIndex(): number;
  free(): void;
}

interface WasmTextWithCursors {
  text(): string;
  cursors(): WasmCursorPosition[];
  free(): void;
}

interface WasmSpanWithHistory {
  text(): string;
  history(): History;
  free(): void;
}

interface WasmTextWithCursorsAndHistory extends WasmTextWithCursors {
  history(): WasmSpanWithHistory[];
}

/**
 * The generated wasm-bindgen surface this library wraps, plus a hook to make
 * sure the underlying module is ready. Supplied by a platform entrypoint.
 */
export interface WasmBackend<
  Cursor extends WasmCursorPosition,
  Text extends WasmTextWithCursors,
> {
  CursorPosition: new (id: number, position: number) => Cursor;
  TextWithCursors: new (text: string, cursors: Cursor[]) => Text;
  reconcile(
    original: string,
    left: Text,
    right: Text,
    tokenizer: BuiltinTokenizer
  ): WasmTextWithCursors;
  reconcileWithHistory(
    original: string,
    left: Text,
    right: Text,
    tokenizer: BuiltinTokenizer
  ): WasmTextWithCursorsAndHistory;
  diff(
    original: string,
    changed: Text,
    tokenizer: BuiltinTokenizer
  ): Array<number | bigint | string>;
  undiff(
    original: string,
    changes: Array<number | bigint | string>,
    tokenizer: BuiltinTokenizer
  ): string;
  /**
   * Make the wasm module ready for use. Invoked before every operation, so it
   * must be cheap and idempotent (a no-op once initialised).
   */
  ensureReady(): void;
}

// Define the enum values as a const array to avoid duplication
const BUILTIN_TOKENIZERS = ['Character', 'Line', 'Markdown', 'Word'] as const;

/**
 * Tokenisation strategies for text merging.
 *
 * These correspond to the built-in tokenizers available in the underlying WASM module.
 */
export type BuiltinTokenizer = (typeof BUILTIN_TOKENIZERS)[number];

/**
 * History classification for text spans in merge results.
 *
 * Indicates the origin of each text span in the merged document.
 */
export type History =
  | 'Unchanged'
  | 'AddedFromLeft'
  | 'AddedFromRight'
  | 'RemovedFromLeft'
  | 'RemovedFromRight';

/**
 * Represents a text document with associated cursor positions.
 *
 * This interface is used both as input to reconcile functions (to specify where
 * cursors are positioned in the original documents) and as output (with cursors
 * automatically repositioned after merging).
 */
export interface TextWithCursors {
  /** The document's entire content as a string */
  text: string;

  /**
   * Array of cursor positions within the text. Can be empty if there are no cursors to track.
   * Each cursor has a unique ID and position.
   */
  cursors: CursorPosition[];
}

/**
 * Like `TextWithCursors`, but cursors may be null or undefined (treated as empty).
 * Used as input where cursor tracking is optional.
 */
export interface TextWithOptionalCursors {
  /** The document's entire content as a string */
  text: string;

  /**
   * Array of cursor positions within the text. Can be null, undefined, or empty
   * if there are no cursors to track. Each cursor has a unique ID and position.
   */
  cursors: null | undefined | CursorPosition[];
}

/**
 * Represents a cursor position within a text document.
 *
 * Cursors are automatically repositioned during text merging to maintain their
 * relative positions as text is inserted, deleted, or modified around them.
 */
export interface CursorPosition {
  /** Unique identifier for the cursor (can be any number, must be unique within the document) */
  id: number;

  /**
   * Zero-based offset in Unicode scalars (code points), from 0 through
   * `[...text].length` inclusive. Applies to both input and output cursors.
   * Non-integer or out-of-range input positions throw `RangeError`.
   * These are not UTF-16 code units or grapheme clusters: `😀` counts as one.
   * Convert DOM selectionStart/selectionEnd offsets before passing them in:
   * `[...text.slice(0, offset)].length`.
   */
  position: number;
}

/**
 * Represents a merged text document with cursor positions and detailed change history.
 *
 * This is the return type of `reconcileWithHistory()` and provides complete information
 * about how the merge was performed, including which parts of the final text came from
 * which source documents.
 */
export interface TextWithCursorsAndHistory {
  /** The merged document's entire content */
  text: string;

  /**
   * Array of cursor positions within the merged text. Can be empty if there are no cursors to track.
   * All cursors are automatically repositioned from the left and right documents.
   */
  cursors: CursorPosition[];

  /**
   * Detailed provenance information showing the origin of each text span in the result.
   * Each span indicates whether it was unchanged, added from left, added from right, etc.
   */
  history: SpanWithHistory[];
}

/**
 * Represents a span of text in the merged result with its change history.
 *
 * This shows exactly which source document contributed each piece of text to the
 * final merged result. Useful for understanding merge decisions and creating
 * visualisations of how documents were combined.
 */
export interface SpanWithHistory {
  /** The text content of this span */
  text: string;

  /** The origin of this text span in the merge result */
  history: History;
}

/** The public, synchronous API surface, identical across platforms. */
export interface ReconcileApi {
  /**
   * Merges three versions of text using intelligent conflict resolution.
   *
   * This is the primary function for 3-way text merging. Unlike traditional merge tools
   * that produce conflict markers, this function automatically resolves conflicts by
   * applying both sets of changes where possible.
   *
   * @param original - The original/base version of the text that both sides diverged from
   * @param left - The left version of the text (either string or TextWithCursors with cursor positions)
   * @param right - The right version of the text (either string or TextWithCursors with cursor positions)
   * @param tokenizer - The tokenisation strategy: "Word" (default, recommended for prose),
   *                    "Character" (fine-grained), "Line" (similar to git merge), or
   *                    "Markdown" (splits on Markdown structure)
   * @returns The reconciled text with automatically repositioned cursor positions
   *
   * @example
   * ```typescript
   * const original = "Hello world";
   * const left = "Hello beautiful world";    // Added "beautiful"
   * const right = "Hi world";                // Changed "Hello" to "Hi"
   *
   * const result = reconcile(original, left, right);
   * console.log(result.text); // "Hi beautiful world"
   * ```
   */
  reconcile(
    original: string,
    left: string | TextWithOptionalCursors,
    right: string | TextWithOptionalCursors,
    tokenizer?: BuiltinTokenizer
  ): TextWithCursors;

  /**
   * Generates a compact diff representation between an original and changed text.
   *
   * These can be parsed and unpacked using the `undiff` function or the Rust crate's EditedText::from_diff.
   * Cursor positions are omitted from the diff result.
   *
   * This function computes the differences between two versions of text and returns
   * a compact representation of those changes.
   *
   * @param original - The original/base version of the text
   * @param changed - The modified version of the text (either string or TextWithCursors with cursor positions)
   * @param tokenizer - The tokenisation strategy, which is the same as used in `reconcile`.
   * @returns An array of inserts (strings), deletes (negative integers), and retained spans (positive integers).
   */
  diff(
    original: string,
    changed: string | TextWithOptionalCursors,
    tokenizer?: BuiltinTokenizer
  ): Array<number | string>;

  /**
   * Applies a compact diff to an original text to reconstruct the changed version.
   *
   * This function takes an original text and a compact diff representation (as produced
   * by the `diff` function) and reconstructs the modified text.
   *
   * @param original - The original/base version of the text
   * @param diff - The compact diff array (inserts as strings, deletes as negative integers, retained spans as positive integers)
   * @param tokenizer - The tokenisation strategy, which is the same as used in `reconcile`.
   * @returns The reconstructed changed text as a string.
   */
  undiff(
    original: string,
    diff: Array<number | bigint | string>,
    tokenizer?: BuiltinTokenizer
  ): string;

  /**
   * Merges three versions of text and returns detailed provenance information.
   *
   * This function behaves like `reconcile()` but also provides
   * detailed historical information about the origin of each text span in the result.
   * This is valuable for understanding how the merge was performed and which changes
   * came from which source.
   *
   * Note: Computing the history is computationally more expensive than the basic merge.
   *
   * @param original - The original/base version of the text that both sides diverged from
   * @param left - The left version of the text (either string or TextWithCursors with cursor positions)
   * @param right - The right version of the text (either string or TextWithCursors with cursor positions)
   * @param tokenizer - The tokenisation strategy: "Word" (default, recommended for prose),
   *                    "Character" (fine-grained), "Line" (similar to git merge), or
   *                    "Markdown" (splits on Markdown structure)
   * @returns The reconciled text with cursor positions and detailed change history
   *
   * @example
   * ```typescript
   * const original = "Hello world";
   * const left = "Hello beautiful world";
   * const right = "Hi world";
   *
   * const result = reconcileWithHistory(original, left, right);
   * console.log(result.text); // "Hi beautiful world"
   * console.log(result.history); // Array of SpanWithHistory objects showing change origins
   * ```
   */
  reconcileWithHistory(
    original: string,
    left: string | TextWithOptionalCursors,
    right: string | TextWithOptionalCursors,
    tokenizer?: BuiltinTokenizer
  ): TextWithCursorsAndHistory;
}

const UNSUPPORTED_TOKENIZER_ERROR = `Unsupported tokenizer, only ${BUILTIN_TOKENIZERS.join(
  ', '
)} are supported`;

/**
 * Build the public {@link ReconcileApi} on top of a {@link WasmBackend}.
 *
 * Each operation calls `backend.ensureReady()` first, then marshals JS values
 * into the wasm representation, invokes the binding, and frees the wasm-side
 * objects. The behaviour is identical regardless of whether the backend is a
 * real WebAssembly module or its wasm2js translation.
 */
export function makeReconcileApi<
  Cursor extends WasmCursorPosition,
  Text extends WasmTextWithCursors,
>(backend: WasmBackend<Cursor, Text>): ReconcileApi {
  function assertTokenizer(tokenizer: BuiltinTokenizer): void {
    if (!BUILTIN_TOKENIZERS.includes(tokenizer)) {
      throw new Error(UNSUPPORTED_TOKENIZER_ERROR);
    }
  }

  function toWasmTextWithCursors(text: string | TextWithOptionalCursors) {
    const isInputString = typeof text === 'string';
    const innerText = isInputString ? text : text.text;
    const innerCursors = isInputString ? [] : (text.cursors ?? []);

    const wasmCursors: Cursor[] = [];
    let textLength: number | undefined;
    try {
      for (const { id, position } of innerCursors) {
        if (textLength === undefined) {
          textLength = 0;
          for (const _character of innerText) {
            textLength++;
          }
        }

        // The WASM constructor panics after consuming the inputs on an invalid
        // position. A trap cannot unwind and release those Rust allocations.
        if (!Number.isInteger(position) || position < 0 || position > textLength) {
          throw new RangeError(
            `Cursor position ${position} must be an integer between 0 and ${textLength}`
          );
        }

        wasmCursors.push(new backend.CursorPosition(id, position));
      }

      return new backend.TextWithCursors(innerText, wasmCursors);
    } catch (error) {
      for (const cursor of wasmCursors) {
        // wasm-bindgen zeroes the pointer when the text constructor takes
        // ownership. Only cursors still owned by JS can be freed here.
        if (!('__wbg_ptr' in cursor) || cursor.__wbg_ptr !== 0) {
          cursor.free();
        }
      }

      throw error;
    }
  }

  function toTextWithCursors(textWithCursor: WasmTextWithCursors): TextWithCursors {
    const wasmCursors = textWithCursor.cursors();
    try {
      const cursors = wasmCursors.map((cursor) => ({
        id: cursor.id(),
        position: cursor.characterIndex(),
      }));

      return {
        text: textWithCursor.text(),
        cursors,
      };
    } finally {
      for (const cursor of wasmCursors) {
        cursor.free();
      }
    }
  }

  function toSpansWithHistory(spans: WasmSpanWithHistory[]): SpanWithHistory[] {
    try {
      return spans.map((span) => ({
        text: span.text(),
        history: span.history(),
      }));
    } finally {
      for (const span of spans) {
        span.free();
      }
    }
  }

  function reconcile(
    original: string,
    left: string | TextWithOptionalCursors,
    right: string | TextWithOptionalCursors,
    tokenizer: BuiltinTokenizer = 'Word'
  ): TextWithCursors {
    backend.ensureReady();
    assertTokenizer(tokenizer);

    const leftCursor = toWasmTextWithCursors(left);
    let rightCursor: Text | undefined;
    let result: WasmTextWithCursors | undefined;
    try {
      rightCursor = toWasmTextWithCursors(right);
      result = backend.reconcile(original, leftCursor, rightCursor, tokenizer);

      return toTextWithCursors(result);
    } finally {
      result?.free();
      rightCursor?.free();
      leftCursor.free();
    }
  }

  function diff(
    original: string,
    changed: string | TextWithOptionalCursors,
    tokenizer: BuiltinTokenizer = 'Word'
  ): Array<number | string> {
    backend.ensureReady();
    assertTokenizer(tokenizer);

    const changedWasm = toWasmTextWithCursors(changed);
    try {
      const result = backend.diff(original, changedWasm, tokenizer);
      return result.map((item) => (typeof item === 'bigint' ? Number(item) : item));
    } finally {
      changedWasm.free();
    }
  }

  function undiff(
    original: string,
    diffValue: Array<number | bigint | string>,
    tokenizer: BuiltinTokenizer = 'Word'
  ): string {
    backend.ensureReady();
    assertTokenizer(tokenizer);

    // The real-WebAssembly backend's `diff` emits BigInt spans, whereas the
    // wasm2js (React Native) backend rejects BigInt outright. Normalise to
    // plain numbers - exactly as `diff` does on the way out - so a `diff`
    // result round-trips through `undiff` identically on every platform.
    return backend.undiff(
      original,
      diffValue.map((item) => (typeof item === 'bigint' ? Number(item) : item)),
      tokenizer
    );
  }

  function reconcileWithHistory(
    original: string,
    left: string | TextWithOptionalCursors,
    right: string | TextWithOptionalCursors,
    tokenizer: BuiltinTokenizer = 'Word'
  ): TextWithCursorsAndHistory {
    backend.ensureReady();
    assertTokenizer(tokenizer);

    const leftCursor = toWasmTextWithCursors(left);
    let rightCursor: Text | undefined;
    let result: WasmTextWithCursorsAndHistory | undefined;
    try {
      rightCursor = toWasmTextWithCursors(right);
      result = backend.reconcileWithHistory(original, leftCursor, rightCursor, tokenizer);

      const jsResult = toTextWithCursors(result);
      const history = toSpansWithHistory(result.history());

      return {
        ...jsResult,
        history,
      };
    } finally {
      result?.free();
      rightCursor?.free();
      leftCursor.free();
    }
  }

  return { reconcile, diff, undiff, reconcileWithHistory };
}
