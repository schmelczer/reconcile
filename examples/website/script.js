import init, { mergeText, mergeTextWithHistory } from "./reconcile.js";

const originalTextArea = document.getElementById("original");
const leftTextArea = document.getElementById("left");
const rightTextArea = document.getElementById("right");
const mergedTextArea = document.getElementById("merged");

const sampleTexts = [
    "The quick brown fox jumps over the lazy dog.",
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
    "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium.",
    "A journey of a thousand miles begins with a single step.",
    "To be, or not to be, that is the question.",
];

async function run() {
    await init();

    originalTextArea.addEventListener("input", updateMergedText);
    leftTextArea.addEventListener("input", updateMergedText);
    rightTextArea.addEventListener("input", updateMergedText);

    loadSample();
    updateMergedText();

    // Put cursor at the end of the text in leftTextArea
    leftTextArea.focus();
    leftTextArea.selectionStart = leftTextArea.value.length;
    leftTextArea.selectionEnd = leftTextArea.value.length;
}

function updateMergedText() {
    const original = originalTextArea.value;
    const left = leftTextArea.value;
    const right = rightTextArea.value;

    const results = mergeTextWithHistory(original, left, right);

    mergedTextArea.innerHTML = "";

    for (const result of results) {
        const span = document.createElement("span");
        span.className = result.history();
        span.textContent = result.text();
        mergedTextArea.appendChild(span);
        result.free();
    }
}

function loadSample() {
    const randomIndex = Math.floor(Math.random() * sampleTexts.length);
    const text = sampleTexts[randomIndex];
    originalTextArea.value = text;
    leftTextArea.value = text;
    rightTextArea.value = text;
}

run();
