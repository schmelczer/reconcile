import init, { mergeText } from "./reconcile.js";

const originalTextArea = document.getElementById("original");
const leftTextArea = document.getElementById("left");
const rightTextArea = document.getElementById("right");
const mergedTextArea = document.getElementById("merged");
const mergeButton = document.getElementById("merge-button");

const sampleTexts = [
    "The quick brown fox jumps over the lazy dog.",
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
    "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium.",
    "A journey of a thousand miles begins with a single step.",
    "To be, or not to be, that is the question.",
];

async function run() {
    await init();

    mergeButton.addEventListener("click", () => {
        const original = originalTextArea.value;
        const left = leftTextArea.value;
        const right = rightTextArea.value;

        const result = mergeText(original, left, right);
        mergedTextArea.value = result;
    });

    loadSample();
}

function loadSample() {
    const randomIndex = Math.floor(Math.random() * sampleTexts.length);
    const text = sampleTexts[randomIndex];
    originalTextArea.value = text;
    leftTextArea.value = text;
    rightTextArea.value = text;
    mergedTextArea.value = "";
}

run();
