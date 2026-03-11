use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use reconcile_text::{
    BuiltinTokenizer, CursorPosition, EditedText, NumberOrText, TextWithCursors,
};

fn parse_tokenizer(tokenizer: &str) -> PyResult<BuiltinTokenizer> {
    match tokenizer {
        "Character" => Ok(BuiltinTokenizer::Character),
        "Line" => Ok(BuiltinTokenizer::Line),
        "Markdown" => Ok(BuiltinTokenizer::Markdown),
        "Word" => Ok(BuiltinTokenizer::Word),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown tokenizer '{tokenizer}', expected Character, Line, Markdown, or Word"
        ))),
    }
}

fn extract_text_with_cursors(input: &Bound<'_, PyAny>) -> PyResult<TextWithCursors> {
    if let Ok(text) = input.extract::<String>() {
        return Ok(TextWithCursors::from(text));
    }

    let dict = input.cast::<PyDict>()?;

    let text: String = dict
        .get_item("text")?
        .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("text"))?
        .extract()?;

    let cursors = match dict.get_item("cursors")? {
        Some(obj) if !obj.is_none() => {
            let list = obj.cast::<PyList>()?;
            let mut cursors = Vec::with_capacity(list.len());
            for item in list {
                let cursor_dict = item.cast::<PyDict>()?;
                let id: usize = cursor_dict
                    .get_item("id")?
                    .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("id"))?
                    .extract()?;
                let position: usize = cursor_dict
                    .get_item("position")?
                    .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("position"))?
                    .extract()?;
                cursors.push(CursorPosition::new(id, position));
            }
            cursors
        }
        _ => Vec::new(),
    };

    Ok(TextWithCursors::new(text, cursors))
}

fn text_with_cursors_to_dict<'py>(
    py: Python<'py>,
    twc: &TextWithCursors,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("text", twc.text())?;

    let cursors = PyList::new(
        py,
        twc.cursors().iter().map(|c| {
            let d = PyDict::new(py);
            d.set_item("id", c.id()).unwrap();
            d.set_item("position", c.char_index()).unwrap();
            d
        }),
    )?;
    dict.set_item("cursors", cursors)?;

    Ok(dict)
}

/// Merge three versions of text using conflict-free resolution.
///
/// Takes a parent text and two concurrent edits (left and right), returning
/// the merged result with automatically repositioned cursors.
///
/// Args:
///     parent: The original text that both sides diverged from.
///     left: The left edit, either a string or a dict with "text" and "cursors" keys.
///     right: The right edit, either a string or a dict with "text" and "cursors" keys.
///     tokenizer: Tokenization strategy - "Word" (default), "Character", "Line", or "Markdown".
///
/// Returns:
///     A dict with "text" (merged string) and "cursors" (list of repositioned cursors).
#[pyfunction]
#[pyo3(signature = (parent, left, right, tokenizer = "Word"))]
fn reconcile<'py>(
    py: Python<'py>,
    parent: &str,
    left: &Bound<'py, PyAny>,
    right: &Bound<'py, PyAny>,
    tokenizer: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let tokenizer = parse_tokenizer(tokenizer)?;
    let left = extract_text_with_cursors(left)?;
    let right = extract_text_with_cursors(right)?;

    let result = reconcile_text::reconcile(parent, &left, &right, &*tokenizer).apply();
    text_with_cursors_to_dict(py, &result)
}

/// Merge three versions of text and return provenance history.
///
/// Like `reconcile`, but also returns which source each text span came from.
///
/// Args:
///     parent: The original text that both sides diverged from.
///     left: The left edit, either a string or a dict with "text" and "cursors" keys.
///     right: The right edit, either a string or a dict with "text" and "cursors" keys.
///     tokenizer: Tokenization strategy - "Word" (default), "Character", "Line", or "Markdown".
///
/// Returns:
///     A dict with "text", "cursors", and "history" (list of dicts with "text" and "history" keys).
#[pyfunction]
#[pyo3(signature = (parent, left, right, tokenizer = "Word"))]
fn reconcile_with_history<'py>(
    py: Python<'py>,
    parent: &str,
    left: &Bound<'py, PyAny>,
    right: &Bound<'py, PyAny>,
    tokenizer: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let tokenizer = parse_tokenizer(tokenizer)?;
    let left = extract_text_with_cursors(left)?;
    let right = extract_text_with_cursors(right)?;

    let reconciled = reconcile_text::reconcile(parent, &left, &right, &*tokenizer);
    let (text_with_cursors, history_spans) = reconciled.apply_with_all();

    let dict = text_with_cursors_to_dict(py, &text_with_cursors)?;

    let history = PyList::new(
        py,
        history_spans.iter().map(|span| {
            let d = PyDict::new(py);
            d.set_item("text", span.text()).unwrap();
            d.set_item("history", format!("{:?}", span.history()))
                .unwrap();
            d
        }),
    )?;
    dict.set_item("history", history)?;

    Ok(dict)
}

/// Generate a compact diff between two texts.
///
/// Returns a list of retain counts (positive ints), delete counts (negative ints),
/// and inserted strings.
///
/// Args:
///     parent: The original text.
///     changed: The modified text, either a string or a dict with "text" and "cursors" keys.
///     tokenizer: Tokenization strategy - "Word" (default), "Character", "Line", or "Markdown".
///
/// Returns:
///     A list of ints and strings representing the diff.
///
/// Raises:
///     ValueError: If the diff computation overflows.
#[pyfunction]
#[pyo3(signature = (parent, changed, tokenizer = "Word"))]
fn diff<'py>(
    py: Python<'py>,
    parent: &str,
    changed: &Bound<'py, PyAny>,
    tokenizer: &str,
) -> PyResult<Bound<'py, PyList>> {
    let tokenizer = parse_tokenizer(tokenizer)?;
    let changed = extract_text_with_cursors(changed)?;

    let edited = EditedText::from_strings_with_tokenizer(parent, &changed, &*tokenizer);
    let diff_result = edited
        .to_diff()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let list = PyList::empty(py);
    for item in diff_result {
        match item {
            NumberOrText::Number(n) => list.append(n)?,
            NumberOrText::Text(s) => list.append(s)?,
        }
    }

    Ok(list)
}

/// Apply a compact diff to a parent text to reconstruct the changed version.
///
/// Args:
///     parent: The original text.
///     diff: A list of ints and strings (as produced by `diff`).
///     tokenizer: Tokenization strategy - "Word" (default), "Character", "Line", or "Markdown".
///
/// Returns:
///     The reconstructed text.
///
/// Raises:
///     ValueError: If the diff format is invalid.
#[pyfunction]
#[pyo3(signature = (parent, diff, tokenizer = "Word"))]
fn undiff(parent: &str, diff: &Bound<'_, PyList>, tokenizer: &str) -> PyResult<String> {
    let tokenizer = parse_tokenizer(tokenizer)?;

    let mut parsed: Vec<NumberOrText> = Vec::with_capacity(diff.len());
    for item in diff {
        if let Ok(n) = item.extract::<i64>() {
            parsed.push(NumberOrText::Number(n));
        } else if let Ok(s) = item.extract::<String>() {
            parsed.push(NumberOrText::Text(s));
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Diff items must be int or str",
            ));
        }
    }

    EditedText::from_diff(parent, parsed, &*tokenizer)
        .map(|edited| edited.apply().text())
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(reconcile, m)?)?;
    m.add_function(wrap_pyfunction!(reconcile_with_history, m)?)?;
    m.add_function(wrap_pyfunction!(diff, m)?)?;
    m.add_function(wrap_pyfunction!(undiff, m)?)?;
    Ok(())
}
