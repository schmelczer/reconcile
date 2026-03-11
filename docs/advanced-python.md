# Advanced Usage (Python)

## Edit Provenance

Track which changes came from where using `reconcile_with_history`:

```python
from reconcile_text import reconcile_with_history

result = reconcile_with_history(
    "Hello world",
    "Hello beautiful world",
    "Hi world",
)

print(result["text"])     # "Hi beautiful world"
print(result["history"])  #
# [
#   {"text": "Hello", "history": "RemovedFromRight"},
#   {"text": "Hi", "history": "AddedFromRight"},
#   {"text": " beautiful", "history": "AddedFromLeft"},
#   {"text": " ", "history": "Unchanged"},
#   {"text": "world", "history": "Unchanged"},
# ]
```

## Tokenization Strategies

`reconcile-text` offers different approaches to split text for merging:

- **Word tokenizer** (`"Word"`) - Splits on word boundaries (recommended for prose)
- **Character tokenizer** (`"Character"`) - Individual characters (fine-grained control)
- **Line tokenizer** (`"Line"`) - Line-by-line (similar to `git merge` or more precisely [`git merge-file`](https://git-scm.com/docs/git-merge-file))
- **Markdown tokenizer** (`"Markdown"`) - Splits on Markdown structural boundaries (headings, list items, paragraphs)

```python
from reconcile_text import reconcile

result = reconcile("abc", "axc", "abyc", "Character")
print(result["text"])  # "axyc"
```

## Cursor Tracking

`reconcile-text` automatically tracks cursor positions through merges, which is useful for collaborative editors. Selections can be tracked by providing them as a pair of cursors.

```python
from reconcile_text import reconcile

result = reconcile(
    "Hello world",
    {
        "text": "Hello beautiful world",
        "cursors": [{"id": 1, "position": 6}],  # After "Hello "
    },
    {
        "text": "Hi world",
        "cursors": [{"id": 2, "position": 0}],  # At the beginning
    },
)

# Result: "Hi beautiful world" with repositioned cursors
print(result["text"])     # "Hi beautiful world"
print(result["cursors"])  # [{"id": 2, "position": 0}, {"id": 1, "position": 3}]
```

> The `cursors` list is sorted by character position (not IDs).

## Compact Diffs

Generate and apply compact diff representations:

```python
from reconcile_text import diff, undiff

original = "Hello world"
changed = "Hello beautiful world"

# Generate a compact diff
d = diff(original, changed)
print(d)  # [5, ' beautiful world']

# Reconstruct the changed text from the diff
reconstructed = undiff(original, d)
assert reconstructed == changed
```

Diff entries are positive integers (retain N characters), negative integers (delete N characters), and strings (insert text).
