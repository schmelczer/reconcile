import {
  CursorPosition as wasmCursorPosition,
  reconcile as wasmReconcile,
  TextWithCursors as wasmTextWithCursors,
  SpanWithHistory as wasmSpanWithHistory,
  reconcileWithHistory as wasmReconcileWithHistory,
  isBinary as wasmIsBinary,
  getCompactDiff as wasmGetCompactDiff,
  initSync,
} from 'reconcile-text';

import wasmBytes from 'reconcile-text/reconcile_text_bg.wasm';

// Define the enum values as const arrays to avoid duplication
const BUILTIN_TOKENIZERS = ['Character', 'Line', 'Word'] as const;
const HISTORY_VALUES = [
  'Unchanged',
  'AddedFromLeft',
  'AddedFromRight',
  'RemovedFromLeft',
  'RemovedFromRight',
] as const;

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
export type History = (typeof HISTORY_VALUES)[number];

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
 * Represents a text document with associated cursor positions.
 *
 * This interface is used both as input to reconcile functions (to specify where
 * cursors are positioned in the original documents) and as output (with cursors
 * automatically repositioned after merging).
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

  /** Character position in the text, 0-based index from the beginning of the document */
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
   * Array of cursor positions within the merged text. Can empty if there are no cursors to track.
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

const UNSUPPORTED_TOKENIZER_ERROR = `Unsupported tokenizer. Only ${BUILTIN_TOKENIZERS.join(
  ', '
)} are supported.`;

let isInitialised = false;

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
 *                    "Character" (fine-grained), or "Line" (similar to git merge)
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
export function reconcile(
  original: string,
  left: string | TextWithOptionalCursors,
  right: string | TextWithOptionalCursors,
  tokenizer: BuiltinTokenizer = 'Word'
): TextWithCursors {
  init();

  if (!BUILTIN_TOKENIZERS.includes(tokenizer)) {
    throw new Error(UNSUPPORTED_TOKENIZER_ERROR);
  }

  const leftCursor = toWasmTextWithCursors(left);
  const rightCursor = toWasmTextWithCursors(right);

  const result = wasmReconcile(original, leftCursor, rightCursor, tokenizer);

  leftCursor.free();
  rightCursor.free();

  const jsResult = toTextWithCursors(result);
  result.free();

  return jsResult;
}

export function getCompactDiff(
  original: string,
  changed: string | TextWithOptionalCursors,
  tokenizer: BuiltinTokenizer = 'Word'
): string {
  init();

  if (!BUILTIN_TOKENIZERS.includes(tokenizer)) {
    throw new Error(UNSUPPORTED_TOKENIZER_ERROR);
  }

  const changedWasm = toWasmTextWithCursors(changed);

  const result = wasmGetCompactDiff(original, changedWasm, tokenizer);

  changedWasm.free();

  return result;
}

/**
 * Merges three versions of text and returns detailed provenance information.
 *
 * This function behaves identically to `reconcile()` but additionally provides
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
 *                    "Character" (fine-grained), or "Line" (similar to git merge)
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
export function reconcileWithHistory(
  original: string,
  left: string | TextWithOptionalCursors,
  right: string | TextWithOptionalCursors,
  tokenizer: BuiltinTokenizer = 'Word'
): TextWithCursorsAndHistory {
  init();

  if (!BUILTIN_TOKENIZERS.includes(tokenizer)) {
    throw new Error(UNSUPPORTED_TOKENIZER_ERROR);
  }

  const leftCursor = toWasmTextWithCursors(left);
  const rightCursor = toWasmTextWithCursors(right);

  const result = wasmReconcileWithHistory(original, leftCursor, rightCursor, tokenizer);

  leftCursor.free();
  rightCursor.free();

  const jsResult = toTextWithCursors(result);
  const history = result.history().map(toSpanWithHistory);
  result.free();

  return {
    ...jsResult,
    history,
  };
}

/**
 * Check (using heuristics) if the given data is binary or text content.
 *
 * Only text inputs can be reconciled using the library's functions.
 *
 * @param data - The data to check for binary content. This should be a Uint8Array.
 * @returns True if the data is likely binary, false if it is likely text.
 */
export function isBinary(data: Uint8Array): boolean {
  init();
  return wasmIsBinary(data);
}

function init() {
  if (isInitialised) {
    return;
  }

  const wasmBinary = Uint8Array.from(atob(wasmBytes as unknown as string), (c) =>
    c.charCodeAt(0)
  );
  initSync({ module: wasmBinary });

  isInitialised = true;
}

function toWasmTextWithCursors(
  text: string | TextWithOptionalCursors
): wasmTextWithCursors {
  const isInputString = typeof text === 'string';
  const leftText = isInputString ? text : text.text;
  const leftCursors = isInputString ? [] : (text.cursors ?? []);

  return new wasmTextWithCursors(leftText, leftCursors.map(toWasmCursorPosition));
}

function toWasmCursorPosition({ id, position }: CursorPosition): wasmCursorPosition {
  return new wasmCursorPosition(id, position);
}

function toTextWithCursors(textWithCursor: wasmTextWithCursors): TextWithCursors {
  return {
    text: textWithCursor.text(),
    cursors: textWithCursor.cursors().map(toCursorPosition),
  };
}

function toCursorPosition(cursor: wasmCursorPosition): CursorPosition {
  return {
    id: cursor.id(),
    position: cursor.characterIndex(),
  };
}

function toSpanWithHistory(textWithHistory: wasmSpanWithHistory): SpanWithHistory {
  return {
    text: textWithHistory.text(),
    history: textWithHistory.history(),
  };
}
