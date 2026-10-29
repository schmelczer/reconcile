from reconcile_text._types import (
    BuiltinTokenizer,
    TextInput,
    TextWithCursors,
    TextWithCursorsAndHistory,
)

def reconcile(
    parent: str,
    left: TextInput,
    right: TextInput,
    tokenizer: BuiltinTokenizer = "Word",
) -> TextWithCursors: ...
def reconcile_with_history(
    parent: str,
    left: TextInput,
    right: TextInput,
    tokenizer: BuiltinTokenizer = "Word",
) -> TextWithCursorsAndHistory: ...
def diff(
    parent: str,
    changed: TextInput,
    tokenizer: BuiltinTokenizer = "Word",
) -> list[int | str]: ...
def undiff(
    parent: str,
    diff: list[int | str],
    tokenizer: BuiltinTokenizer = "Word",
) -> str: ...
