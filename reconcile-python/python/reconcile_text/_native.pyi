from typing import Any

def reconcile(
    parent: str,
    left: Any,
    right: Any,
    tokenizer: str = "Word",
) -> dict[str, Any]: ...
def reconcile_with_history(
    parent: str,
    left: Any,
    right: Any,
    tokenizer: str = "Word",
) -> dict[str, Any]: ...
def diff(
    parent: str,
    changed: Any,
    tokenizer: str = "Word",
) -> list[int | str]: ...
def undiff(
    parent: str,
    diff: list[int | str],
    tokenizer: str = "Word",
) -> str: ...
