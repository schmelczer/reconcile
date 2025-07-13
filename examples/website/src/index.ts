import { reconcile, reconcileWithHistory } from 'reconcile-text';
import type { BuiltinTokenizer } from 'reconcile-text';
import './style.scss';

const originalTextArea = document.getElementById('original') as HTMLTextAreaElement;
const leftTextArea = document.getElementById('left') as HTMLTextAreaElement;
const rightTextArea = document.getElementById('right') as HTMLTextAreaElement;
const mergedTextArea = document.getElementById('merged') as HTMLDivElement;
const tokenizerRadios = document.querySelectorAll(
  'input[name="tokenizer"]'
) as NodeListOf<HTMLInputElement>;

const sampleText = `The "reconcile-text" Rust library is embedded on this page as a WASM module and powers these text boxes. Experiment with changing the "Original", "First user's edit", and "Second user's edit" text boxes to see competing changes get merged in real-time within the "Merged result" box. Here, you will see color-coded tokens marking the origin of each token, including ones that got deleted. The result highly depends on the tokenisation strategy, for example, deciding how casing or whitespace is taken into account.`;

async function main(): Promise<void> {
  originalTextArea.addEventListener('input', updateMergedText);
  leftTextArea.addEventListener('input', updateMergedText);
  rightTextArea.addEventListener('input', updateMergedText);

  leftTextArea.addEventListener('selectionchange', updateMergedText);
  rightTextArea.addEventListener('selectionchange', updateMergedText);
  leftTextArea.addEventListener('select', updateMergedText);
  rightTextArea.addEventListener('select', updateMergedText);

  console.info(
    reconcile(
      'Hello world',
      {
        text: 'Hello beautiful world',
        cursors: [{ id: 1, position: 6 }], // After "Hello "
      },
      {
        text: 'Hi world',
        cursors: [{ id: 2, position: 0 }], // At the beginning
      }
    )
  );

  window.addEventListener('resize', resizeTextAreas);

  tokenizerRadios.forEach((radio) => {
    radio.addEventListener('change', updateMergedText);
  });

  loadSample();
  updateMergedText();
  focusTextArea(leftTextArea);
}

// Edit the instructions to generate example edits
function loadSample(): void {
  originalTextArea.value = sampleText;
  leftTextArea.value =
    sampleText.replace('color', 'colour') +
    " Check out what's the most complex conflict you can come up with!";
  rightTextArea.value = sampleText
    .replace(', for example,', ' such as')
    .replace('WASM', 'WebAssembly');
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

  let selectionStart: number = Number.NEGATIVE_INFINITY;
  let selectionEnd: number = Number.NEGATIVE_INFINITY;
  if (results.cursors?.length ?? 0 > 0) {
    selectionStart = results.cursors![0].position;
    selectionEnd = results.cursors![1].position;
  }
  const isSelection = selectionStart !== selectionEnd;

  const selectionSide = leftCursors ? 'left' : 'right';
  const fragment = document.createDocumentFragment();

  let currentPosition = 0;
  if (selectionEnd === 0) {
    fragment.appendChild(createSelectionOverlay(selectionSide === 'left', isSelection));
  }

  for (const { text, history } of results.history) {
    for (const character of text) {
      const span = document.createElement('span');
      span.className = history;
      span.textContent = character;

      if (selectionStart <= currentPosition && currentPosition < selectionEnd) {
        span.className += ` selection-${selectionSide}`;
      }

      fragment.appendChild(span);

      const isDelete = history === 'RemovedFromLeft' || history === 'RemovedFromRight';
      if (currentPosition === selectionEnd - 1 && !isDelete) {
        fragment.appendChild(
          createSelectionOverlay(selectionSide === 'left', isSelection)
        );
      }

      if (!isDelete) {
        // Only increment currentPosition for non-removed characters
        currentPosition++;
      }
    }
  }

  mergedTextArea.innerHTML = '';
  mergedTextArea.appendChild(fragment);
}

function getCursorsFromActiveTextArea() {
  const activeElement = document.activeElement;
  let leftCursors = undefined;
  let rightCursors = undefined;

  if (activeElement === leftTextArea) {
    leftCursors = [
      { id: 1, position: leftTextArea.selectionStart },
      { id: 2, position: leftTextArea.selectionEnd },
    ];
  } else if (activeElement === rightTextArea) {
    rightCursors = [
      { id: 1, position: rightTextArea.selectionStart },
      { id: 2, position: rightTextArea.selectionEnd },
    ];
  }
  return { leftCursors, rightCursors };
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
  return selectedRadio?.value as BuiltinTokenizer;
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
  textarea.style.height = textarea.scrollHeight + 'px';
}

function focusTextArea(textarea: HTMLTextAreaElement): void {
  textarea.focus();
  textarea.selectionStart = 0;
  textarea.selectionEnd = 0;
}

main();
