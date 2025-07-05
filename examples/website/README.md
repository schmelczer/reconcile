# Reconcile: Interactive Demo

This is the interactive demo website for the Reconcile library. Visit [schmelczer.dev/reconcile](https://schmelczer.dev/reconcile) to try it out.

## About the Demo

The demo allows you to:

- Enter three text versions (parent, left, right)
- See the reconciled result in real-time
- Experiment with different tokenization strategies
- Observe how cursor positions are updated during merging
- View the history of operations that led to the result

## Features Demonstrated

- **Conflict-free merging**: No conflict markers in the output
- **Cursor tracking**: See how cursor positions are automatically updated
- **Different tokenizers**: Compare word-level vs. character-level tokenization
- **Operation history**: Understand the merge process step-by-step

## Running Locally

```bash
# Build the WASM module first
cd ../..
wasm-pack build --target web

# Install dependencies and run the demo
cd examples/website
npm install
npm run dev
```

## Usage Examples

Try these examples in the demo:

### Basic merge
- **Parent**: "Hello world"
- **Left**: "Hello beautiful world"
- **Right**: "Hi world"
- **Result**: "Hi beautiful world"

### Cursor tracking
- **Parent**: "The quick brown fox"
- **Left**: "The very quick brown fox" (cursor at position 4)
- **Right**: "The quick red fox" (cursor at position 10)
- **Result**: Cursors automatically repositioned

### Character-level merging
Switch to character tokenizer for fine-grained merging of individual characters rather than whole words.

For more examples and detailed documentation, see the [main README](../../README.md).
