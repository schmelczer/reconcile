import { reconcileWithHistory } from 'reconcile';
import type { Tokenizer } from 'reconcile';
import './style.scss';

const originalTextArea = document.getElementById('original') as HTMLTextAreaElement;
const leftTextArea = document.getElementById('left') as HTMLTextAreaElement;
const rightTextArea = document.getElementById('right') as HTMLTextAreaElement;
const mergedTextArea = document.getElementById('merged') as HTMLDivElement;
const tokenizerRadios = document.querySelectorAll(
  'input[name="tokenizer"]'
) as NodeListOf<HTMLInputElement>;

const sampleText = `The \`reconcile\` Rust library is embedded on this page as a WASM module and powers these text boxes. Experiment with changing the "Original", "First concurrent edit", and "Second concurrent edit" text boxes to see competing changes get merged in real-time within the "Deconflicted result" box. Here, you will see color-coded tokens marking the origin of each token, including ones that got deleted. The result highly depends on the tokenization strategy, for example, deciding how casing or whitespace is taken into account.`;

async function main(): Promise<void> {
  originalTextArea.addEventListener('input', updateMergedText);
  leftTextArea.addEventListener('input', updateMergedText);
  rightTextArea.addEventListener('input', updateMergedText);
  window.addEventListener('resize', resizeTextAreas);

  tokenizerRadios.forEach((radio) => {
    radio.addEventListener('change', updateMergedText);
  });

  loadSample();
  updateMergedText();
  focusTextArea(leftTextArea);
}

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

  const results = reconcileWithHistory(original, left, right, selectedTokenizer);

  mergedTextArea.innerHTML = '';

  for (const { text, history } of results.history) {
    const span = document.createElement('span');
    span.className = history;
    span.textContent = text;
    mergedTextArea.appendChild(span);
  }
}

function getSelectedTokenizer(): Tokenizer {
  const selectedRadio = Array.from(tokenizerRadios).find((radio) => radio.checked);
  return selectedRadio?.value as Tokenizer;
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
  textarea.selectionStart = textarea.value.length;
  textarea.selectionEnd = textarea.value.length;
}

main();
