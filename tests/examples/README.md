# Test Examples

This directory contains YAML test cases that demonstrate various reconcile scenarios.

## Format

Each YAML file contains test documents with the following structure:

```yaml
parent: "Original text"
left: 
  text: "Left version"
  cursors:
    - id: 1
      char_index: 5
right:
  text: "Right version"  
  cursors:
    - id: 2
      char_index: 10
expected:
  text: "Expected result"
  cursors:
    - id: 1
      char_index: 8
    - id: 2
      char_index: 12
```

## Cursor Position Notation

In some test cases, the `|` character is used to denote cursor positions within the text. These characters are stripped before the actual reconcile logic is run, making it easier to visualize where cursors should be positioned.

## Running Tests

These examples are automatically tested as part of the test suite:

```bash
cargo test
```

The tests verify that:
1. Text is merged correctly without conflicts
2. Cursor positions are updated accurately
3. The merge result is consistent regardless of argument order (left/right swap)
