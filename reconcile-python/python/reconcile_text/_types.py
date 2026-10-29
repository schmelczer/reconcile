"""Public input and result types."""

from __future__ import annotations

from typing import Literal, TypedDict, Union

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
