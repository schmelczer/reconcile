use super::{token::Token, word_tokenizer::split_words};

/// Splits markdown text into tokens that respect markdown formatting structure
///
/// Builds on word-level tokenization with markdown-specific handling:
/// - Newlines are non-joinable tokens (preserves block structure)
/// - Block-level prefixes (headings, list markers, blockquotes) attach to the
///   first word of their line so they can't be split apart during merge
/// - Intra-line whitespace uses the same normalization as the word tokenizer
///
/// This prevents merges from breaking lists, headings, or other structural
/// markdown elements. Inline formatting like `**bold**` is already preserved
/// by word-level splitting since formatting markers contain no whitespace.
///
/// ## Example
///
/// ```not_rust
/// "# Hello\n- item" -> ["# Hello", "\n", "- item"]
/// ```
pub fn markdown_tokenizer(text: &str) -> Vec<Token<String>> {
    let mut result = Vec::new();
    let segments = split_preserving_newlines(text);

    for segment in &segments {
        if *segment == "\n" || *segment == "\r\n" {
            let s = (*segment).to_owned();
            result.push(Token::new(s.clone(), s, false, false));
            continue;
        }

        let prefix_len = block_prefix_len(segment);
        let mut line_tokens = split_words(&segment[prefix_len..]);

        if prefix_len > 0 {
            let prefix = &segment[..prefix_len];
            if line_tokens.is_empty() {
                let s = prefix.to_owned();
                result.push(Token::new(s.clone(), s, false, false));
            } else {
                let first = &line_tokens[0];
                let combined_original = format!("{prefix}{}", first.original());
                let combined_normalized = format!("{prefix}{}", first.normalized());
                line_tokens[0] = Token::new(
                    combined_normalized,
                    combined_original,
                    false,
                    first.is_right_joinable,
                );
            }
        }

        result.extend(line_tokens);
    }

    // Normalize non-newline whitespace tokens by appending the next token's
    // original text (same trick as the word tokenizer so each space is unique
    // in the diff based on what follows it)
    if !result.is_empty() {
        for i in 0..result.len() - 1 {
            if result[i]
                .original()
                .chars()
                .all(|c| c.is_whitespace() && c != '\n' && c != '\r')
            {
                let normalized = result[i].normalized().to_owned() + result[i + 1].original();
                result[i].set_normalized(normalized);
            }
        }
    }

    result
}

/// Splits text into alternating segments of line content and newline separators
fn split_preserving_newlines(text: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut line_start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            if i > line_start {
                segments.push(&text[line_start..i]);
            }
            segments.push(&text[i..i + 2]);
            i += 2;
            line_start = i;
        } else if bytes[i] == b'\n' {
            if i > line_start {
                segments.push(&text[line_start..i]);
            }
            segments.push(&text[i..=i]);
            i += 1;
            line_start = i;
        } else {
            i += 1;
        }
    }

    if line_start < text.len() {
        segments.push(&text[line_start..]);
    }

    segments
}

/// Returns the byte length of a markdown block-level prefix at the start of a
/// line, or 0 if none is found
///
/// All recognized prefix characters are ASCII, so byte offsets are always
/// valid UTF-8 boundaries.
///
/// Recognized prefixes:
/// - ATX headings: `# ` through `###### `
/// - Blockquotes: `> ` (single level)
/// - Unordered lists: `- `, `* `, `+ ` (with optional leading whitespace)
/// - Ordered lists: `1. `, `2) ` etc (with optional leading whitespace)
/// - Task lists: `- [ ] `, `- [x] `, `- [X] ` etc (checkbox included in prefix)
fn block_prefix_len(line: &str) -> usize {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indent_len = line.len() - trimmed.len();

    // ATX heading: #{1,6} followed by a space
    if trimmed.starts_with('#') {
        let hash_count = trimmed.bytes().take_while(|&b| b == b'#').count();
        if hash_count <= 6 && trimmed.as_bytes().get(hash_count) == Some(&b' ') {
            return indent_len + hash_count + 1;
        }
    }

    // Blockquote: > followed by optional space
    if trimmed.starts_with("> ") {
        return indent_len + 2;
    }
    if trimmed.starts_with('>') && (trimmed.len() == 1 || trimmed.as_bytes()[1] == b'>') {
        return indent_len + 1;
    }

    // Unordered list: [-*+] followed by a space, optionally with task checkbox
    if trimmed.len() >= 2 {
        let first_byte = trimmed.as_bytes()[0];
        if matches!(first_byte, b'-' | b'*' | b'+') && trimmed.as_bytes()[1] == b' ' {
            return indent_len + 2 + task_checkbox_len(&line[indent_len + 2..]);
        }
    }

    // Ordered list: digits followed by [.)] and a space, optionally with task
    // checkbox
    let digit_count = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if digit_count > 0 && indent_len + digit_count + 2 <= line.len() {
        let after_digits = trimmed.as_bytes()[digit_count];
        let after_marker = trimmed.as_bytes().get(digit_count + 1);
        if matches!(after_digits, b'.' | b')') && after_marker == Some(&b' ') {
            return indent_len
                + digit_count
                + 2
                + task_checkbox_len(&line[indent_len + digit_count + 2..]);
        }
    }

    0
}

/// Returns the byte length of a task list checkbox (`[ ] `, `[x] `, `[X] `)
/// at the start of `rest`, or 0 if none is found
fn task_checkbox_len(rest: &str) -> usize {
    if rest.len() >= 4
        && rest.as_bytes()[0] == b'['
        && matches!(rest.as_bytes()[1], b' ' | b'x' | b'X')
        && rest.as_bytes()[2] == b']'
        && rest.as_bytes()[3] == b' '
    {
        4
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_plain_text() {
        assert_debug_snapshot!(markdown_tokenizer("Hello world"));
    }

    #[test]
    fn test_empty() {
        assert_debug_snapshot!(markdown_tokenizer(""));
    }

    #[test]
    fn test_headings() {
        assert_debug_snapshot!(markdown_tokenizer("# Hello world"));
        assert_debug_snapshot!(markdown_tokenizer("## Sub heading"));
        assert_debug_snapshot!(markdown_tokenizer("###### Deep heading"));
    }

    #[test]
    fn test_unordered_list() {
        assert_debug_snapshot!(markdown_tokenizer("- item one\n- item two\n- item three"));
    }

    #[test]
    fn test_ordered_list() {
        assert_debug_snapshot!(markdown_tokenizer("1. first\n2. second\n3. third"));
    }

    #[test]
    fn test_blockquote() {
        assert_debug_snapshot!(markdown_tokenizer("> quoted text\n> more quoted"));
    }

    #[test]
    fn test_inline_formatting() {
        assert_debug_snapshot!(markdown_tokenizer("Some **bold** and *italic* text"));
    }

    #[test]
    fn test_mixed_content() {
        assert_debug_snapshot!(markdown_tokenizer(
            "# Title\n\nSome text with **bold**.\n\n- list item\n- another item"
        ));
    }

    #[test]
    fn test_indented_list() {
        assert_debug_snapshot!(markdown_tokenizer("  - nested item\n    - deeper"));
    }

    #[test]
    fn test_crlf() {
        assert_debug_snapshot!(markdown_tokenizer("Line 1\r\nLine 2"));
    }

    #[test]
    fn test_code_fence() {
        assert_debug_snapshot!(markdown_tokenizer("```rust\nlet x = 1;\n```"));
    }

    #[test]
    fn test_heading_only() {
        assert_debug_snapshot!(markdown_tokenizer("# "));
    }

    #[test]
    fn test_link() {
        assert_debug_snapshot!(markdown_tokenizer("Click [here](https://example.com) now"));
    }

    #[test]
    fn test_multiline_paragraph() {
        assert_debug_snapshot!(markdown_tokenizer(
            "First line\nSecond line\n\nNew paragraph"
        ));
    }

    #[test]
    fn test_list_with_star_marker() {
        assert_debug_snapshot!(markdown_tokenizer("* item one\n* item two"));
    }

    #[test]
    fn test_bold_not_confused_with_list() {
        assert_debug_snapshot!(markdown_tokenizer("**bold text**"));
    }

    #[test]
    fn test_task_list() {
        assert_debug_snapshot!(markdown_tokenizer(
            "- [ ] todo\n- [x] done\n- [X] also done"
        ));
    }

    #[test]
    fn test_ordered_task_list() {
        assert_debug_snapshot!(markdown_tokenizer("1. [ ] first task\n2. [x] second task"));
    }

    #[test]
    fn test_unicode() {
        assert_debug_snapshot!(markdown_tokenizer(
            "# \u{1F600} Héllo\n- \u{00E9}lément\n> \u{4F60}\u{597D} world"
        ));
    }
}
