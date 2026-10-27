"""Merge three versions of a file: mine, base, and theirs.

A trivial version of git merge-file (https://git-scm.com/docs/git-merge-file).

Run it with:
    uv run --directory reconcile-python \
        python ../examples/merge_file.py my.txt base.txt their.txt [output.txt]
"""

from __future__ import annotations

import sys
from pathlib import Path

from reconcile_text import reconcile


def main() -> None:
    args = sys.argv[1:]

    if len(args) < 3 or len(args) > 4:
        print("Usage: merge_file.py <mine> <base> <theirs> [output]", file=sys.stderr)
        sys.exit(1)

    # Disable universal newline translation to preserve the original line endings.
    with Path(args[0]).open(newline="") as file:
        mine = file.read()
    with Path(args[1]).open(newline="") as file:
        base = file.read()
    with Path(args[2]).open(newline="") as file:
        theirs = file.read()

    result = reconcile(base, mine, theirs)

    if len(args) == 4:
        with Path(args[3]).open("w", newline="") as file:
            file.write(result["text"])
    else:
        reconfigure = getattr(sys.stdout, "reconfigure", None)
        if reconfigure is not None:
            reconfigure(newline="")
        print(result["text"], end="")


if __name__ == "__main__":
    main()
