#![cfg(feature = "wasm")]

use reconcile_text::{BuiltinTokenizer, CursorPosition, TextWithCursors, wasm::*};
use wasm_bindgen_test::*;

#[wasm_bindgen_test(unsupported = test)]
fn test_merge() {
    let left = b"hello ";
    let right = b"world";
    let result = generic_reconcile(b"", left, right, BuiltinTokenizer::Word);
    assert_eq!(result, b"hello world");

    let left = b"\0binary";
    let right = b"other";
    let result = generic_reconcile(b"", left, right, BuiltinTokenizer::Word);
    assert_eq!(result, right);
}

#[wasm_bindgen_test(unsupported = test)]
fn test_merge_text() {
    let left = "hello ";
    let right = "world";
    let result = reconcile("", &left.into(), &right.into(), BuiltinTokenizer::Word).text();
    assert_eq!(result, "hello world");
}

#[wasm_bindgen_test(unsupported = test)]
fn test_merge_text_with_cursors() {
    let result = reconcile(
        "hi",
        &TextWithCursors::new("hi world".to_owned(), vec![]),
        &TextWithCursors::new(
            "hi".to_owned(),
            vec![CursorPosition::new(0, 1), CursorPosition::new(1, 2)],
        ),
        BuiltinTokenizer::Word,
    );

    assert_eq!(
        result,
        TextWithCursors::new(
            "hi world".to_owned(),
            vec![CursorPosition::new(0, 1), CursorPosition::new(1, 2)]
        ),
    );
}

#[wasm_bindgen_test(unsupported = test)]
fn merge_binary() {
    let left = [0, 1, 2];
    let right = [3, 4, 5];
    assert_eq!(
        generic_reconcile(b"", &left, &right, BuiltinTokenizer::Word),
        right
    );
}

#[wasm_bindgen_test(unsupported = test)]
fn test_is_binary() {
    assert!(is_binary(&[0, 159, 146, 150]));
    assert!(is_binary(&[0, 12]));
    assert!(!is_binary(b"hello"));
}

#[wasm_bindgen_test(unsupported = test)]
fn test_is_binary_empty() {
    assert!(!is_binary(b""));
}
