import wasmInit, {
  CursorPosition as wasmCursorPosition,
  reconcile as wasmReconcile,
  TextWithCursors as wasmTextWithCursors,
  SpanWithHistory as wasmSpanWithHistory,
  BuiltinTokenizer,
  reconcileWithHistory as wasmReconcileWithHistory,
  History,
  initSync,
} from 'reconcile';

import wasm from 'reconcile/reconcile_bg.wasm';

export interface TextWithCursors {
  /** The document's entire content */
  text: string;
  /** List of cursor positions, can be null or undefined if there are no cursors */
  cursors: null | undefined | CursorPosition[];
}

export interface CursorPosition {
  /** Unique identifier for the cursor */
  id: number;
  /** Character position in the text, 0-based */
  position: number;
}

export interface TextWithCursorsAndHistory {
  /** The document's entire content */
  text: string;
  /** List of cursor positions, can be null or undefined if there are no cursors */
  cursors: null | undefined | CursorPosition[];
  /** List of operations leading to `text` from the 3 ancestors */
  history: SpanWithHistory[];
}

export interface SpanWithHistory {
  /** Span of text associated with the historical opearion */
  text: string;
  /** Origin of the `text` span */
  history: History;
}

export type Tokenizer = 'Line' | 'Word' | 'Character';
const TOKENIZERS = ['Line', 'Word', 'Character'];

let isInitialised = false;

const UNSUPPORTED_TOKENIZER_ERROR = `Unsupported tokenizer. Only ${TOKENIZERS.join(
  ', '
)} are supported.`;

/**
 * Merges three versions of text (original, left, right) and cursor positions.
 *
 * @param original - The original/base version of the text
 * @param left - The left version of the text, either as string or TextWithCursors
 * @param right - The right version of the text, either as string or TextWithCursors
 * @param tokenizer - The tokenization strategy to use (default: "Word")
 * @returns The reconciled text with merged cursor positions
 */
export function reconcile(
  original: string,
  left: string | TextWithCursors,
  right: string | TextWithCursors,
  tokenizer: BuiltinTokenizer = 'Word'
): TextWithCursors {
  init();

  if (!TOKENIZERS.includes(tokenizer)) {
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

/**
 * Merges three versions of text and returns the result with historical information.
 *
 * Calculating the `history` is somewhat more expensive, otherwise this function behaves like `reconcile`.
 *
 * @param original - The original/base version of the text
 * @param left - The left version of the text, either as string or TextWithCursors
 * @param right - The right version of the text, either as string or TextWithCursors
 * @param tokenizer - The tokenization strategy to use (default: "Word")
 * @returns The reconciled text with cursor positions and history of changes
 */
export function reconcileWithHistory(
  original: string,
  left: string | TextWithCursors,
  right: string | TextWithCursors,
  tokenizer: BuiltinTokenizer = 'Word'
): TextWithCursorsAndHistory {
  init();

  if (!TOKENIZERS.includes(tokenizer)) {
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

function init() {
  if (isInitialised) {
    return;
  }

  initSync({ module: (wasm as any).default });

  isInitialised = true;
}

function toWasmTextWithCursors(text: string | TextWithCursors): wasmTextWithCursors {
  const isInputString = typeof text == 'string';
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
