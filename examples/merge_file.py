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

    mine = Path(args[0]).read_text()
    base = Path(args[1]).read_text()
    theirs = Path(args[2]).read_text()

    result = reconcile(base, mine, theirs)

    if len(args) == 4:
        Path(args[3]).write_text(result["text"])
    else:
        print(result["text"], end="")


if __name__ == "__main__":
    main()
