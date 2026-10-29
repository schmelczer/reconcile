import { reconcileWithHistory } from 'reconcile-text';
import type { BuiltinTokenizer, CursorPosition } from 'reconcile-text';
import './style.scss';

const originalTextArea = document.getElementById('original') as HTMLTextAreaElement;
const leftTextArea = document.getElementById('left') as HTMLTextAreaElement;
const rightTextArea = document.getElementById('right') as HTMLTextAreaElement;
const mergedTextArea = document.getElementById('merged') as HTMLDivElement;
const showDeletions = document.getElementById('show-deletions') as HTMLInputElement;
const tokenizerRadios = document.querySelectorAll(
  'input[name="tokenizer"]'
) as NodeListOf<HTMLInputElement>;

const sampleText = `## Draft release
## Release plan
- Prototype
- Publish
- [ ] Draft
- [ ] Review
> Internal
> Ready
Color-coded edits.`;

let pendingUpdate: number | null = null;
function scheduleUpdate(): void {
  if (pendingUpdate === null) {
    pendingUpdate = requestAnimationFrame(() => {
      pendingUpdate = null;
      updateMergedText();
    });
  }
}

async function main(): Promise<void> {
  originalTextArea.addEventListener('input', scheduleUpdate);
  leftTextArea.addEventListener('input', scheduleUpdate);
  rightTextArea.addEventListener('input', scheduleUpdate);
  showDeletions.addEventListener('change', () => {
    mergedTextArea.classList.toggle('show-deletions', showDeletions.checked);
  });

  document.addEventListener('selectionchange', () => {
    if (
      document.activeElement === leftTextArea ||
      document.activeElement === rightTextArea
    ) {
      scheduleUpdate();
    }
  });

  window.addEventListener('resize', resizeTextAreas);

  tokenizerRadios.forEach((radio) => {
    radio.addEventListener('change', scheduleUpdate);
  });

  loadSample();
  updateMergedText();
}

// Deleting obsolete lines while reformatting their neighbours leaves orphaned
// markers with Word, but Markdown keeps each marker attached to its line's first word.
function loadSample(): void {
  originalTextArea.value = sampleText;
  leftTextArea.value = sampleText
    .replace('## Draft release\n', '')
    .replace('- Prototype\n', '')
    .replace('- [ ] Draft\n', '')
    .replace('> Internal\n', '')
    .replace('Color', 'Colour');
  rightTextArea.value = sampleText
    .replace('## Release plan', '# Release plan')
    .replace('- Publish', '1. Publish')
    .replace('- [ ] Review', '- [x] Review')
    .replace('> Ready', 'Ready');
}

function updateMergedText(): void {
  resizeTextAreas();

  const original = originalTextArea.value;
  const left = leftTextArea.value;
  const right = rightTextArea.value;

  const selectedTokenizer = getSelectedTokenizer();

  const { leftCursors, rightCursors } = getCursorsFromActiveTextArea();

  const results = reconcileWithHistory(
    original,
    {
      text: left,
      cursors: leftCursors,
    },
    {
      text: right,
      cursors: rightCursors,
    },
    selectedTokenizer
  );

  const selection =
    results.cursors?.length === 2
      ? {
        start: results.cursors[0].position,
        end: results.cursors[1].position,
        side: leftCursors ? 'left' : 'right',
      }
      : undefined;

  const fragment = document.createDocumentFragment();

  function appendSpan(text: string, className: string): void {
    if (text.length === 0) {
      return;
    }

    const span = document.createElement('span');
    span.className = className;
    span.textContent = text;
    fragment.appendChild(span);
  }

  function appendCaret(): void {
    if (!selection) {
      return;
    }

    fragment.appendChild(
      createSelectionOverlay(
        selection.side === 'left',
        selection.start !== selection.end
      )
    );
  }

  let currentPosition = 0;
  if (selection?.end === 0) {
    appendCaret();
  }

  for (const { text, history } of results.history) {
    const isDelete = history === 'RemovedFromLeft' || history === 'RemovedFromRight';
    if (isDelete || !selection) {
      appendSpan(text, history);
      continue;
    }

    // Reconcile positions count Unicode scalars, rather than UTF-16 code units.
    const chars = [...text];
    const start = Math.max(0, Math.min(chars.length, selection.start - currentPosition));
    const end = Math.max(0, Math.min(chars.length, selection.end - currentPosition));
    appendSpan(chars.slice(0, start).join(''), history);
    appendSpan(
      chars.slice(start, end).join(''),
      `${history} selection-${selection.side}`
    );

    if (
      currentPosition < selection.end &&
      selection.end <= currentPosition + chars.length
    ) {
      appendCaret();
    }

    appendSpan(chars.slice(end).join(''), history);
    currentPosition += chars.length;
  }

  mergedTextArea.innerHTML = '';
  mergedTextArea.appendChild(fragment);
}

function getCursorsFromActiveTextArea() {
  const activeElement = document.activeElement;
  let leftCursors = undefined;
  let rightCursors = undefined;

  if (activeElement === leftTextArea) {
    leftCursors = getSelectionCursors(leftTextArea);
  } else if (activeElement === rightTextArea) {
    rightCursors = getSelectionCursors(rightTextArea);
  }
  return { leftCursors, rightCursors };
}

function getSelectionCursors(textarea: HTMLTextAreaElement): CursorPosition[] {
  const text = textarea.value;
  // DOM selections count UTF-16 code units; reconcile counts Unicode scalars.
  return [
    { id: 1, position: [...text.slice(0, textarea.selectionStart)].length },
    { id: 2, position: [...text.slice(0, textarea.selectionEnd)].length },
  ];
}

function createSelectionOverlay(isLeft: boolean, isSelection: boolean): HTMLSpanElement {
  const caretSpan = document.createElement('span');
  caretSpan.className = `selection-caret selection-caret-${isLeft ? 'left' : 'right'}`;

  const stickDiv = document.createElement('div');
  stickDiv.className = 'stick';
  caretSpan.appendChild(stickDiv);

  const dotDiv = document.createElement('div');
  dotDiv.className = 'dot';
  caretSpan.appendChild(dotDiv);

  const infoDiv = document.createElement('div');
  infoDiv.className = 'info';
  const selectionType = isSelection ? 'selection' : 'cursor';
  infoDiv.textContent = isLeft
    ? `Left user's ${selectionType}`
    : `Right user's ${selectionType}`;
  caretSpan.appendChild(infoDiv);

  return caretSpan;
}

function getSelectedTokenizer(): BuiltinTokenizer {
  const selectedRadio = Array.from(tokenizerRadios).find((radio) => radio.checked);
  return (selectedRadio?.value ?? 'Word') as BuiltinTokenizer;
}

function resizeTextAreas(): void {
  // Only auto-resize if field-sizing CSS property is not supported, like in Safari as of now
  if (!CSS.supports('field-sizing', 'content')) {
    autoResize(originalTextArea);
    autoResize(leftTextArea);
    autoResize(rightTextArea);
  }
}

function autoResize(textarea: HTMLTextAreaElement): void {
  textarea.style.height = 'auto';
  const borderHeight = textarea.offsetHeight - textarea.clientHeight;
  textarea.style.height = textarea.scrollHeight + borderHeight + 'px';
}

main().catch((error) => {
  document.body.textContent =
    'Failed to load the application. Please ensure your browser supports WebAssembly.';
  console.error(error);
});
