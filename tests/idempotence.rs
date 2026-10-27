use reconcile_text::{
    BuiltinTokenizer, CursorPosition, EditedText, TextWithCursors, Token, reconcile,
};
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

#[test_matrix([
    BuiltinTokenizer::Character,
    BuiltinTokenizer::Line,
    BuiltinTokenizer::Markdown,
    BuiltinTokenizer::Word
])]
fn replay_short_edits_preserves_merged_text(tokenizer: BuiltinTokenizer) {
    let mut texts = vec![String::new()];
    let mut level = texts.clone();
    for _ in 0..2 {
        level = level
            .iter()
            .flat_map(|text| ['a', 'b', ' ', '\n'].map(|ch| format!("{text}{ch}")))
            .collect();
        texts.extend(level.iter().cloned());
    }

    for parent in &texts {
        for left in &texts {
            for right in &texts {
                let merged = reconcile(parent, &left.into(), &right.into(), &*tokenizer).apply();
                assert_eq!(
                    reconcile(parent, &right.into(), &left.into(), &*tokenizer)
                        .apply()
                        .text(),
                    merged.text(),
                    "swapped inputs: tokenizer={tokenizer:?}, parent={parent:?}, left={left:?}, right={right:?}",
                );
                for replayed in [left, right] {
                    let actual = reconcile(parent, &merged, &replayed.into(), &*tokenizer).apply();
                    assert_eq!(
                        actual.text(),
                        merged.text(),
                        "tokenizer={tokenizer:?}, parent={parent:?}, left={left:?}, right={right:?}, replayed={replayed:?}"
                    );
                }
            }
        }
    }
}

#[test_matrix(["", "a"], [
    BuiltinTokenizer::Character,
    BuiltinTokenizer::Line,
    BuiltinTokenizer::Markdown,
    BuiltinTokenizer::Word
])]
fn replay_conflicting_replacements_keeps_both_edits(parent: &str, tokenizer: BuiltinTokenizer) {
    let left = TextWithCursors::new("b".to_owned(), vec![CursorPosition::new(1, 1)]);
    let right = TextWithCursors::new("x".to_owned(), vec![CursorPosition::new(2, 1)]);
    let merged = reconcile(parent, &left, &right, &*tokenizer).apply();
    assert_eq!(merged.text(), "bx");
    assert_eq!(
        merged.cursors(),
        vec![CursorPosition::new(1, 1), CursorPosition::new(2, 2)]
    );
    for input in ["b", "x"] {
        let replayed = reconcile(parent, &merged, &input.into(), &*tokenizer).apply();
        assert_eq!(replayed, merged);
    }
    let left_operations = EditedText::from_strings_with_tokenizer(parent, &left, &*tokenizer);
    let right_operations = EditedText::from_strings_with_tokenizer(parent, &right, &*tokenizer);
    assert_eq!(
        left_operations
            .merge_with_tokenizer(&right_operations, &*tokenizer)
            .apply(),
        merged
    );
}

#[test_matrix([
    BuiltinTokenizer::Character,
    BuiltinTokenizer::Line,
    BuiltinTokenizer::Markdown,
    BuiltinTokenizer::Word
])]
fn replay_preserves_repeated_text_and_unicode_cursors(tokenizer: BuiltinTokenizer) {
    let merged = reconcile("a", &"bb".into(), &"x".into(), &*tokenizer).apply();
    assert_eq!(merged.text(), "bbx");
    assert_eq!(
        reconcile("a", &merged, &"x".into(), &*tokenizer).apply(),
        merged
    );

    let left = TextWithCursors::new(
        "🦊x".to_owned(),
        vec![CursorPosition::new(1, 1), CursorPosition::new(2, 2)],
    );
    let right = TextWithCursors::new("x".to_owned(), vec![CursorPosition::new(3, 1)]);
    let edited = reconcile("a", &left, &right, &*tokenizer);
    let result = edited.apply();
    assert_eq!(result.text(), "🦊x");
    for (id, position) in [(1, 1), (2, 2), (3, 2)] {
        assert!(
            result
                .cursors()
                .contains(&CursorPosition::new(id, position))
        );
    }
    let history_text: String = edited
        .apply_with_history()
        .iter()
        .filter(|span| {
            !matches!(
                span.history(),
                reconcile_text::History::RemovedFromLeft
                    | reconcile_text::History::RemovedFromRight
            )
        })
        .map(reconcile_text::SpanWithHistory::text)
        .collect();
    assert_eq!(history_text, result.text());
    assert_eq!(edited.apply_with_all().0, result);
}

#[test_matrix([
    BuiltinTokenizer::Character,
    BuiltinTokenizer::Line,
    BuiltinTokenizer::Markdown,
    BuiltinTokenizer::Word
])]
fn replay_longer_mixed_edits_is_stable(tokenizer: BuiltinTokenizer) {
    let mut seed = 0x12ab_34cd_56ef_7890u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let alphabet = ['a', 'b', 'c', ' ', '\n', '\r', 'é', '🦊'];
    for case in 0..2000 {
        let mut text = || {
            (0..next() % 16)
                .map(|_| alphabet[usize::try_from(next() % 8).unwrap()])
                .collect::<String>()
        };
        let parent = text();
        let left = text();
        let right = text();
        let merged = reconcile(
            &parent,
            &left.clone().into(),
            &right.clone().into(),
            &*tokenizer,
        )
        .apply();
        if case < 200 {
            let left_with_cursors = TextWithCursors::new(
                left.clone(),
                (0..=left.chars().count())
                    .map(|position| CursorPosition::new(position, position))
                    .collect(),
            );
            let right_with_cursors = TextWithCursors::new(
                right.clone(),
                (0..=right.chars().count())
                    .map(|position| CursorPosition::new(100 + position, position))
                    .collect(),
            );
            let result = reconcile(
                &parent,
                &left_with_cursors,
                &right_with_cursors,
                &*tokenizer,
            )
            .apply();
            assert_eq!(result.text(), merged.text());
            assert_eq!(
                result.cursors().len(),
                left_with_cursors.cursors().len() + right_with_cursors.cursors().len()
            );
            assert!(
                result
                    .cursors()
                    .iter()
                    .all(|cursor| cursor.char_index() <= result.text().chars().count())
            );
            for cursor in left_with_cursors
                .cursors()
                .into_iter()
                .chain(right_with_cursors.cursors())
            {
                assert!(
                    result
                        .cursors()
                        .iter()
                        .any(|updated| updated.id() == cursor.id())
                );
            }
        }
        for input in [&left, &right] {
            assert_eq!(
                reconcile(&parent, &merged, &input.into(), &*tokenizer)
                    .apply()
                    .text(),
                merged.text(),
                "tokenizer={tokenizer:?}, parent={parent:?}, left={left:?}, right={right:?}, input={input:?}"
            );
        }
        assert_eq!(
            reconcile(&parent, &right.into(), &left.into(), &*tokenizer)
                .apply()
                .text(),
            merged.text()
        );
    }
}

#[test]
fn replay_with_normalized_tokens_preserves_spelling() {
    let tokenizer = |text: &str| {
        text.chars()
            .map(|ch| Token::new(ch.to_ascii_lowercase(), ch.to_string(), false, false))
            .collect()
    };
    let texts = ["", "A", "a", "AB", "ba", "Aa", "a A", " A", "\nA"];
    for parent in texts {
        for left in texts {
            for right in texts {
                let merged = reconcile(parent, &left.into(), &right.into(), &tokenizer).apply();
                for input in [left, right] {
                    assert_eq!(
                        reconcile(parent, &merged, &input.into(), &tokenizer)
                            .apply()
                            .text(),
                        merged.text()
                    );
                }
            }
        }
    }
}

#[test]
fn ambiguous_ancestral_text_prefers_a_stable_input() {
    let parent = "a b ";
    let left = "a b a b ".into();
    let right = "c d ".into();
    let tokenizer = &*BuiltinTokenizer::Word;
    let result = reconcile(parent, &left, &right, tokenizer);
    let merged = result.apply();
    assert_eq!(merged.text(), "a b a b ");
    for input in [&left, &right] {
        assert_eq!(
            reconcile(parent, &merged, input, tokenizer).apply().text(),
            merged.text()
        );
    }
    assert!(
        result
            .apply_with_history()
            .iter()
            .any(|span| span.history() == reconcile_text::History::AddedFromLeft)
    );
    assert!(
        !result
            .apply_with_history()
            .iter()
            .any(|span| span.history() == reconcile_text::History::AddedFromRight)
    );
}
