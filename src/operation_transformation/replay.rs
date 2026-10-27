use std::fmt::Debug;

use super::EditedText;
use crate::{BuiltinTokenizer, Side, TextWithCursors, Tokenizer};

pub(super) fn merge_stable<'a, T>(
    original: &'a str,
    left: &TextWithCursors,
    right: &TextWithCursors,
    tokenizer: &Tokenizer<T>,
) -> EditedText<'a, T>
where
    T: PartialEq + Clone + Debug,
{
    // A combined result is accepted only if the candidate merge absorbs both
    // inputs. On replay the candidate is then the same result, and its checks
    // are unchanged or compare identical snapshots. If this fails, selecting
    // one input is also stable: replay sees the original pair or identical inputs.
    let candidate = merge_candidate(original, left, right, tokenizer);
    if left.text_ref() == right.text_ref()
        || left.text_ref() == original
        || right.text_ref() == original
    {
        return candidate;
    }
    let merged = candidate.apply().text();
    let without_cursors = merged.clone().into();
    if [left, right].iter().all(|input| {
        merge_candidate(
            original,
            &without_cursors,
            &input.text_ref().into(),
            tokenizer,
        )
        .apply()
        .text()
            == merged
    }) {
        return candidate;
    }

    let left_operations = EditedText::from_strings_with_tokenizer(original, left, tokenizer);
    let right_operations = EditedText::from_strings_with_tokenizer(original, right, tokenizer);
    if version_priority(&left_operations, left) >= version_priority(&right_operations, right) {
        keep_version(left_operations, right_operations, left, right, Side::Left)
    } else {
        keep_version(right_operations, left_operations, right, left, Side::Right)
    }
}

fn is_subsequence(part: &str, whole: &str) -> bool {
    let mut chars = whole.chars();
    part.chars()
        .all(|ch| chars.by_ref().any(|candidate| candidate == ch))
}

fn version_priority<'b, T>(
    operations: &EditedText<'_, T>,
    input: &'b TextWithCursors,
) -> (usize, usize, &'b str)
where
    T: PartialEq + Clone + Debug,
{
    let (inserted, deleted) = operations.edit_counts();
    (inserted, deleted, input.text_ref())
}

fn merge_candidate<'a, T>(
    original: &'a str,
    left: &TextWithCursors,
    right: &TextWithCursors,
    tokenizer: &Tokenizer<T>,
) -> EditedText<'a, T>
where
    T: PartialEq + Clone + Debug,
{
    let left_operations = EditedText::from_strings_with_tokenizer(original, left, tokenizer);
    let right_operations = EditedText::from_strings_with_tokenizer(original, right, tokenizer);
    let left_text = left.text_ref();
    let right_text = right.text_ref();
    if left_text == right_text || left_text == original || right_text == original {
        return merge_ordered(left_operations, right_operations, left_text, right_text);
    }

    let left_inserted = left_operations.inserted_text();
    let right_inserted = right_operations.inserted_text();
    // Treat insertions already present in the other side's edits as incorporated,
    // including inside joined words or across separate inserted spans. This
    // deliberately gives replay precedence over ambiguous independent edits.
    let left_contains_right =
        !right_inserted.is_empty() && is_subsequence(&right_inserted, &left_inserted);
    let right_contains_left =
        !left_inserted.is_empty() && is_subsequence(&left_inserted, &right_inserted);
    match (left_contains_right, right_contains_left) {
        (true, false) => keep_version(left_operations, right_operations, left, right, Side::Left),
        (false, true) => keep_version(right_operations, left_operations, right, left, Side::Right),
        (true, true) => {
            if version_priority(&left_operations, left)
                >= version_priority(&right_operations, right)
            {
                keep_version(left_operations, right_operations, left, right, Side::Left)
            } else {
                keep_version(right_operations, left_operations, right, left, Side::Right)
            }
        }
        (false, false) => merge_ordered(left_operations, right_operations, left_text, right_text),
    }
}

fn merge_ordered<'a, T>(
    left: EditedText<'a, T>,
    right: EditedText<'a, T>,
    left_text: &str,
    right_text: &str,
) -> EditedText<'a, T>
where
    T: PartialEq + Clone + Debug,
{
    // Canonical input order also stabilizes tokenizers with normalized or
    // non-joinable tokens. Restore provenance to the caller's original sides.
    if left_text <= right_text {
        left.merge(right)
    } else {
        right.merge(left).swap_sides()
    }
}

fn keep_version<'a, T>(
    operations: EditedText<'a, T>,
    other_operations: EditedText<'a, T>,
    kept: &TextWithCursors,
    other: &TextWithCursors,
    side: Side,
) -> EditedText<'a, T>
where
    T: PartialEq + Clone + Debug,
{
    let kept_text = kept.text_ref();
    let traditional = merge_ordered(
        operations.clone(),
        other_operations,
        kept_text,
        other.text_ref(),
    );
    // Preserve the existing cursor mapping and provenance when the operation
    // merge already produces the selected text. Character projection is needed
    // only when replay suppression actually changes the operation result.
    if traditional.apply().text_ref() == kept_text {
        return match side {
            Side::Left => traditional,
            Side::Right => traditional.swap_sides(),
        };
    }
    let mut cursors = kept.cursors();
    if !other.cursors().is_empty() {
        let other_text = other.text_ref();
        // Character alignment maps cursors inside joined words and uses the
        // same character offsets as the public cursor API, including Unicode.
        let original = EditedText::from_strings_with_tokenizer(
            other_text,
            other,
            &*BuiltinTokenizer::Character,
        );
        let updated = EditedText::from_strings_with_tokenizer(
            other_text,
            &kept_text.into(),
            &*BuiltinTokenizer::Character,
        );
        cursors.extend(original.merge(updated).apply().cursors());
    }
    operations.with_cursors_and_side(cursors, side)
}
