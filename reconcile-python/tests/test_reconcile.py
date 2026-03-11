from __future__ import annotations

from pathlib import Path

import pytest

from reconcile_text import diff, reconcile, reconcile_with_history, undiff

RESOURCES_DIR = Path(__file__).resolve().parent.parent.parent / "tests" / "resources"

FILES = ["pride_and_prejudice.txt", "room_with_a_view.txt", "blns.txt"]


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
