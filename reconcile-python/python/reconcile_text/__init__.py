"""Intelligent 3-way text merging with automated conflict resolution."""

from reconcile_text._native import diff, reconcile, reconcile_with_history, undiff
from reconcile_text._types import (
    BuiltinTokenizer,
    CursorPosition,
    History,
    SpanWithHistory,
    TextInput,
    TextWithCursors,
    TextWithCursorsAndHistory,
)

__all__ = [
    "BuiltinTokenizer",
    "CursorPosition",
    "History",
    "SpanWithHistory",
    "TextInput",
    "TextWithCursors",
    "TextWithCursorsAndHistory",
    "diff",
    "reconcile",
    "reconcile_with_history",
    "undiff",
]
