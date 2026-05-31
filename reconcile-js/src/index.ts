import {
  CursorPosition as wasmCursorPosition,
  TextWithCursors as wasmTextWithCursors,
  reconcile as wasmReconcile,
  reconcileWithHistory as wasmReconcileWithHistory,
  diff as wasmDiff,
  undiff as wasmUndiff,
  initSync,
} from 'reconcile-text';

import wasmBytes from 'reconcile-text/reconcile_text_bg.wasm';

import { makeReconcileApi, type WasmBackend } from './core';

let isInitialised = false;

const backend: WasmBackend = {
  CursorPosition: wasmCursorPosition,
  TextWithCursors: wasmTextWithCursors,
  reconcile: wasmReconcile,
  reconcileWithHistory: wasmReconcileWithHistory,
  diff: wasmDiff,
  undiff: wasmUndiff,
  ensureReady() {
    if (isInitialised) {
      return;
    }

    const wasmBinary = Uint8Array.from(atob(wasmBytes as unknown as string), (c) =>
      c.charCodeAt(0)
    );
    initSync({ module: wasmBinary });

    isInitialised = true;
  },
};

export const { reconcile, diff, undiff, reconcileWithHistory } =
  makeReconcileApi(backend);

export type {
  BuiltinTokenizer,
  History,
  CursorPosition,
  TextWithCursors,
  TextWithOptionalCursors,
  TextWithCursorsAndHistory,
  SpanWithHistory,
} from './core';
