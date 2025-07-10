# Test Examples

This directory contains comprehensive YAML test cases that demonstrate various text reconciliation scenarios and edge cases. These examples serve both as regression tests and as documentation of the library's behaviour in different situations.

## Test Structure

Each YAML file contains test cases with the following structure:
- `parent`: The original text that both sides diverged from
- `left`: One version of the edited text
- `right`: Another version of the edited text  
- `expected`: The expected merged result
- `description`: Human-readable explanation of what the test demonstrates

## Cursor Position Notation

In some test cases, the `|` character is used to denote cursor positions within the text. These characters are stripped before the actual reconcile logic is run, making it easier to visualise where cursors should be positioned in the test inputs and expected outputs.
