"""Intelligent 3-way text merging with automated conflict resolution."""

from __future__ import annotations

from typing import Literal, TypedDict, Union

from reconcile_text._native import diff as _diff
from reconcile_text._native import reconcile as _reconcile
from reconcile_text._native import reconcile_with_history as _reconcile_with_history
from reconcile_text._native import undiff as _undiff

BuiltinTokenizer = Literal["Character", "Line", "Markdown", "Word"]
"""Tokenization strategy for text merging."""

History = Literal[
    "Unchanged", "AddedFromLeft", "AddedFromRight", "RemovedFromLeft", "RemovedFromRight"
]
"""Provenance label for each span in a merge result."""


class CursorPosition(TypedDict):
    """A cursor position within a text document."""

    id: int
    """Unique identifier for the cursor."""
    position: int
    """Character position in the text (0-based)."""


class TextWithCursors(TypedDict):
    """A text document with associated cursor positions."""

    text: str
    """The document content."""
    cursors: list[CursorPosition]
    """Cursor positions within the text."""


class SpanWithHistory(TypedDict):
    """A text span annotated with its origin in a merge result."""

    text: str
    """The text content of this span."""
    history: History
    """Which source this span came from."""


class TextWithCursorsAndHistory(TypedDict):
    """A merged text document with cursor positions and change provenance."""

    text: str
    """The merged document content."""
    cursors: list[CursorPosition]
    """Repositioned cursor positions."""
    history: list[SpanWithHistory]
    """Provenance information for each text span."""


TextInput = Union[str, TextWithCursors]
"""Input type for text arguments: either a plain string or a dict with text and cursors."""


def reconcile(
    parent: str,
    left: TextInput,
    right: TextInput,
    tokenizer: BuiltinTokenizer = "Word",
) -> TextWithCursors:
    """Merge three versions of text using conflict-free resolution.

    Takes a parent text and two concurrent edits (left and right), returning
    the merged result with automatically repositioned cursors.

    Args:
        parent: The original text that both sides diverged from.
        left: The left edit (string or dict with "text" and "cursors").
        right: The right edit (string or dict with "text" and "cursors").
        tokenizer: Tokenization strategy. Defaults to "Word".

    Returns:
        A dict with "text" (merged string) and "cursors" (repositioned cursor list).
    """
    return _reconcile(parent, left, right, tokenizer)  # type: ignore[return-value]


def reconcile_with_history(
    parent: str,
    left: TextInput,
    right: TextInput,
    tokenizer: BuiltinTokenizer = "Word",
) -> TextWithCursorsAndHistory:
    """Merge three versions of text and return provenance history.

    Like `reconcile`, but also returns which source each text span came from.

    Args:
        parent: The original text that both sides diverged from.
        left: The left edit (string or dict with "text" and "cursors").
        right: The right edit (string or dict with "text" and "cursors").
        tokenizer: Tokenization strategy. Defaults to "Word".

    Returns:
        A dict with "text", "cursors", and "history".
    """
    return _reconcile_with_history(parent, left, right, tokenizer)  # type: ignore[return-value]


def diff(
    parent: str,
    changed: TextInput,
    tokenizer: BuiltinTokenizer = "Word",
) -> list[int | str]:
    """Generate a compact diff between two texts.

    Returns retain counts (positive ints), delete counts (negative ints),
    and inserted strings.

    Args:
        parent: The original text.
        changed: The modified text (string or dict with "text" and "cursors").
        tokenizer: Tokenization strategy. Defaults to "Word".

    Returns:
        A list of ints and strings representing the diff.

    Raises:
        ValueError: If the diff computation overflows.
    """
    return _diff(parent, changed, tokenizer)  # type: ignore[return-value]


def undiff(
    parent: str,
    diff: list[int | str],
    tokenizer: BuiltinTokenizer = "Word",
) -> str:
    """Apply a compact diff to reconstruct the changed text.

    Args:
        parent: The original text.
        diff: A list of ints and strings (as produced by `diff`).
        tokenizer: Tokenization strategy. Defaults to "Word".

    Returns:
        The reconstructed text.

    Raises:
        ValueError: If the diff format is invalid.
    """
    return _undiff(parent, diff, tokenizer)


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
