from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from reconcile_text import TextWithCursors, diff, reconcile, reconcile_with_history, undiff

EXAMPLES_DIR = Path(__file__).resolve().parent.parent.parent / "examples"
RESOURCES_DIR = Path(__file__).resolve().parent.parent.parent / "tests" / "resources"

FILES = ["pride_and_prejudice.txt", "room_with_a_view.txt", "blns.txt"]


class TestCursorPositions:
    @pytest.mark.parametrize("text", ["", "a", "é", "😀", "e\u0301"])
    def test_out_of_bounds(self, text: str) -> None:
        position = len(text) + 1
        invalid: TextWithCursors = {
            "text": text,
            "cursors": [{"id": 1, "position": position}],
        }
        message = f"Cursor position {position} exceeds text length {len(text)}"

        for merge in (reconcile, reconcile_with_history):
            with pytest.raises(ValueError, match=message):
                merge(text, invalid, text)
            with pytest.raises(ValueError, match=message):
                merge(text, text, invalid)

        with pytest.raises(ValueError, match=message):
            diff(text, invalid)

    @pytest.mark.parametrize("text", ["", "a", "é", "😀", "e\u0301"])
    def test_boundary_positions(self, text: str) -> None:
        valid: TextWithCursors = {
            "text": text,
            "cursors": [{"id": 1, "position": 0}, {"id": 2, "position": len(text)}],
        }

        for merge in (reconcile, reconcile_with_history):
            for left, right in ((valid, text), (text, valid)):
                result = merge(text, left, right)
                assert result["text"] == text
                assert result["cursors"] == valid["cursors"]

        assert diff(text, valid) == diff(text, text)


class TestReconcile:
    def test_basic_merge(self) -> None:
        result = reconcile("Hello", "Hello world", "Hi world")
        assert result["text"] == "Hi world"

    def test_three_way_merge(self) -> None:
        parent = "Merging text is hard!"
        left = "Merging text is easy!"
        right = "With reconcile, merging documents is hard!"

        result = reconcile(parent, left, right)
        assert result["text"] == "With reconcile, merging documents is easy!"

    def test_with_cursors(self) -> None:
        result = reconcile(
            "Hello",
            {"text": "Hello world", "cursors": [{"id": 3, "position": 2}]},
            {
                "text": "Hi world",
                "cursors": [{"id": 4, "position": 0}, {"id": 5, "position": 3}],
            },
        )

        assert result["text"] == "Hi world"
        assert result["cursors"] == [
            {"id": 3, "position": 0},
            {"id": 4, "position": 0},
            {"id": 5, "position": 3},
        ]

    def test_character_tokenizer(self) -> None:
        result = reconcile("abc", "axc", "abyc", "Character")
        assert result["text"] == "axyc"

    def test_line_tokenizer(self) -> None:
        parent = "line1\nline2\nline3\n"
        left = "line1\nmodified\nline3\n"
        right = "line1\nline2\nnew line\n"

        result = reconcile(parent, left, right, "Line")
        assert result["text"] == "line1\nmodified\nnew line\n"

    def test_empty_texts(self) -> None:
        result = reconcile("", "", "")
        assert result["text"] == ""
        assert result["cursors"] == []

    def test_invalid_tokenizer(self) -> None:
        with pytest.raises(ValueError, match="Unknown tokenizer"):
            reconcile("a", "b", "c", "Invalid")  # type: ignore[arg-type]


class TestReconcileWithHistory:
    def test_returns_history(self) -> None:
        result = reconcile_with_history(
            "Merging text is hard!",
            "Merging text is easy!",
            "With reconcile, merging documents is hard!",
        )

        assert result["text"] == "With reconcile, merging documents is easy!"
        assert len(result["history"]) > 0
        assert all("text" in span and "history" in span for span in result["history"])

    def test_history_values(self) -> None:
        valid_histories = {
            "Unchanged",
            "AddedFromLeft",
            "AddedFromRight",
            "RemovedFromLeft",
            "RemovedFromRight",
        }
        result = reconcile_with_history("Hello", "Hello world", "Hi")
        for span in result["history"]:
            assert span["history"] in valid_histories


class TestDiff:
    def test_basic_diff(self) -> None:
        result = diff("Hello world", "Hello beautiful world")
        assert isinstance(result, list)
        assert all(isinstance(item, (int, str)) for item in result)

    def test_no_change(self) -> None:
        result = diff("same text", "same text")
        # A retain-only diff
        assert all(isinstance(item, int) and item > 0 for item in result)


class TestUndiff:
    def test_roundtrip(self) -> None:
        original = "Hello world"
        changed = "Hello beautiful world"

        d = diff(original, changed)
        reconstructed = undiff(original, d)
        assert reconstructed == changed

    def test_empty_roundtrip(self) -> None:
        d = diff("", "")
        assert undiff("", d) == ""

    def test_invalid_diff(self) -> None:
        with pytest.raises(ValueError):
            undiff("short", [100])


class TestExamples:
    @pytest.mark.parametrize(
        "content",
        [
            b"line1\r\nline2\r\n",
            b"line1\nline2\n",
            b"line1\rline2\r",
            b"line1\r\nline2\nline3\rline4",
        ],
        ids=["crlf", "lf", "cr", "mixed-no-final-newline"],
    )
    @pytest.mark.parametrize("write_output", [False, True], ids=["stdout", "file"])
    def test_merge_file_preserves_line_endings(
        self, tmp_path: Path, content: bytes, write_output: bool
    ) -> None:
        inputs = [tmp_path / name for name in ("mine.txt", "base.txt", "theirs.txt")]
        for path in inputs:
            path.write_bytes(content)

        args = [sys.executable, str(EXAMPLES_DIR / "merge_file.py"), *map(str, inputs)]
        output = tmp_path / "output.txt"
        if write_output:
            args.append(str(output))

        result = subprocess.run(args, capture_output=True, check=True)

        actual = output.read_bytes() if write_output else result.stdout
        assert actual == content

    def test_merge_file_stdout(self, tmp_path: Path) -> None:
        (tmp_path / "base.txt").write_text("Hello world")
        (tmp_path / "mine.txt").write_text("Hello beautiful world")
        (tmp_path / "theirs.txt").write_text("Hi world")

        result = subprocess.run(
            [
                sys.executable,
                str(EXAMPLES_DIR / "merge_file.py"),
                str(tmp_path / "mine.txt"),
                str(tmp_path / "base.txt"),
                str(tmp_path / "theirs.txt"),
            ],
            capture_output=True,
            text=True,
            check=True,
        )

        assert result.stdout == "Hi beautiful world"

    def test_merge_file_output_file(self, tmp_path: Path) -> None:
        (tmp_path / "base.txt").write_text("Hello world")
        (tmp_path / "mine.txt").write_text("Hello beautiful world")
        (tmp_path / "theirs.txt").write_text("Hi world")
        output = tmp_path / "output.txt"

        subprocess.run(
            [
                sys.executable,
                str(EXAMPLES_DIR / "merge_file.py"),
                str(tmp_path / "mine.txt"),
                str(tmp_path / "base.txt"),
                str(tmp_path / "theirs.txt"),
                str(output),
            ],
            capture_output=True,
            text=True,
            check=True,
        )

        assert output.read_text() == "Hi beautiful world"


class TestDiffUndiffInverse:
    """Verify diff/undiff roundtrip across large real-world texts."""

    @pytest.mark.parametrize("file1", FILES)
    @pytest.mark.parametrize("file2", FILES)
    def test_roundtrip_files(self, file1: str, file2: str) -> None:
        content1 = (RESOURCES_DIR / file1).read_text()[:50000]
        content2 = (RESOURCES_DIR / file2).read_text()[:50000]

        changes = diff(content1, content2)
        actual = undiff(content1, changes)
        assert actual == content2
