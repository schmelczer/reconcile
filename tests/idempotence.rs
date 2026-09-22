use reconcile_text::{BuiltinTokenizer, Token, reconcile};
use test_case::{test_case, test_matrix};

#[test_matrix([
    BuiltinTokenizer::Character,
    BuiltinTokenizer::Line,
    BuiltinTokenizer::Markdown,
    BuiltinTokenizer::Word
])]
fn identical_edits_preserve_text(tokenizer: BuiltinTokenizer) {
    let texts = [
        "",
        "a",
        "b\n",
        "\r\n\r\n",
        "# Héllo\n\n- 世界\n",
        "a a a\nb b b\n",
    ];
    for parent in texts {
        for updated in texts {
            let input = updated.into();
            assert_eq!(
                reconcile(parent, &input, &input, &*tokenizer)
                    .apply()
                    .text(),
                updated,
                "tokenizer={tokenizer:?}, parent={parent:?}, updated={updated:?}",
            );
        }
    }
}

#[test_matrix([
    BuiltinTokenizer::Character,
    BuiltinTokenizer::Line,
    BuiltinTokenizer::Markdown,
    BuiltinTokenizer::Word
])]
fn identical_short_edits_preserve_text(tokenizer: BuiltinTokenizer) {
    let mut texts = vec![String::new()];
    let mut level = texts.clone();
    for _ in 0..3 {
        level = level
            .iter()
            .flat_map(|text| ['a', 'b', ' ', '\n'].map(|ch| format!("{text}{ch}")))
            .collect();
        texts.extend(level.iter().cloned());
    }

    for parent in &texts {
        for updated in &texts {
            let input = updated.into();
            assert_eq!(
                reconcile(parent, &input, &input, &*tokenizer)
                    .apply()
                    .text(),
                *updated,
                "tokenizer={tokenizer:?}, parent={parent:?}, updated={updated:?}",
            );
        }
    }
}

#[test_case("A", "a"; "lowercase")]
#[test_case("a", "A"; "uppercase")]
#[test_case("Hello WORLD", "HELLO world")]
#[test_case("X A", "x a\n")]
#[test_case("A", "new a"; "normalized_suffix")]
#[test_case("a old B", "A new b"; "normalized_prefix_and_suffix")]
fn identical_edits_preserve_original_spelling(parent: &str, updated: &str) {
    let tokenizer = |text: &str| {
        text.chars()
            .map(|ch| Token::new(ch.to_ascii_lowercase(), ch.to_string(), false, false))
            .collect()
    };
    let input = updated.into();
    assert_eq!(
        reconcile(parent, &input, &input, &tokenizer).apply().text(),
        updated,
    );
}
