import { init, reconcileWithHistory } from 'reconcile';
import './style.scss';

const originalTextArea = document.getElementById('original') as HTMLTextAreaElement;
const leftTextArea = document.getElementById('left') as HTMLTextAreaElement;
const rightTextArea = document.getElementById('right') as HTMLTextAreaElement;
const mergedTextArea = document.getElementById('merged') as HTMLDivElement;

const sampleText = `The \`reconcile\` Rust library is embedded on this page a WASM module and it powers these text boxes. Experiment with the "Original", "First concurrent edit", and "Second concurrent edit" text boxes to watch competing changes merge in real-time within the "Deconflicted result" box. Here, you will see color-coded tokens marking the origin of each token, including ones that got deleted. The result highly depends on the tokenization strategy, for example, deciding how casing or white-spacing is taken into account.`;

async function main(): Promise<void> {
  await init();

  originalTextArea?.addEventListener('input', updateMergedText);
  leftTextArea?.addEventListener('input', updateMergedText);
  rightTextArea?.addEventListener('input', updateMergedText);
  window.addEventListener('resize', resizeTextAreas);

  loadSample();
  updateMergedText();
  if (leftTextArea) focusTextArea(leftTextArea);
}

function loadSample(): void {
  if (originalTextArea) originalTextArea.value = sampleText;
  if (leftTextArea) {
    leftTextArea.value =
      sampleText.replace('color', 'colour') +
      " Check out what's the most complex conflict you can come up with!";
  }
  if (rightTextArea) {
    rightTextArea.value = sampleText
      .replace(', for example,', ' such as')
      .replace('WASM', 'WebAssembly');
  }
}

function updateMergedText(): void {
  resizeTextAreas();

  if (!originalTextArea || !leftTextArea || !rightTextArea || !mergedTextArea) {
    return;
  }

  const original = originalTextArea.value;
  const left = leftTextArea.value;
  const right = rightTextArea.value;

  const results = reconcileWithHistory(original, left, right);

  mergedTextArea.innerHTML = '';

  for (const { text, history } of results.history) {
    const span = document.createElement('span');
    span.className = history;
    span.textContent = text;
    mergedTextArea.appendChild(span);
  }
}

function resizeTextAreas(): void {
  if (!CSS.supports('field-sizing', 'content')) {
    if (originalTextArea) autoResize(originalTextArea);
    if (leftTextArea) autoResize(leftTextArea);
    if (rightTextArea) autoResize(rightTextArea);
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
