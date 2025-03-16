use insta::assert_debug_snapshot;
use sync_lib::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test(unsupported = test)]
fn test_bytes_to_base64() {
    let input = b"hello";
    let expected = "aGVsbG8=";
    assert_eq!(bytes_to_base64(input), expected);
}

#[wasm_bindgen_test(unsupported = test)]
fn test_base64_to_bytes() {
    let input = "aGVsbG8=";
    let expected = b"hello".to_vec();
    assert_eq!(base64_to_bytes(input).unwrap(), expected);
}

#[test] // insta doesn't support wasm-bindgen-test
fn test_base64_to_bytes_error() {
    let input = "===";
    assert_debug_snapshot!(base64_to_bytes(input));
}

#[wasm_bindgen_test(unsupported = test)]
fn merge_text() {
    let left = b"hello ";
    let right = b"world";
    let result = merge(b"", left, right);
    assert_eq!(result, b"hello world");
}

#[wasm_bindgen_test(unsupported = test)]
fn merge_binary() {
    let left = [0, 1, 2];
    let right = [3, 4, 5];
    assert_eq!(merge(b"", &left, &right), right);
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

#[wasm_bindgen_test(unsupported = test)]
fn test_is_file_type_mergable() {
    assert!(is_file_type_mergable(".md"));
    assert!(is_file_type_mergable("hi.md"));
    assert!(is_file_type_mergable("my/path/to/my/document.md"));
    assert!(is_file_type_mergable("hi.MD"));
    assert!(is_file_type_mergable("my/path/to/my/DOCUMENT.MD"));

    assert!(!is_file_type_mergable(".json"));
    assert!(!is_file_type_mergable("HELLO.JSON"));
    assert!(!is_file_type_mergable("my/config.yml"));
}
