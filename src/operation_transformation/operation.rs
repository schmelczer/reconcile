use core::fmt::{Debug, Display};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    Token,
    utils::{
        find_longest_prefix_contained_within::find_longest_prefix_contained_within,
        string_builder::StringBuilder,
    },
};

/// Represents a change that can be applied on a `StringBuilder`
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub enum Operation<T>
where
    T: PartialEq + Clone + Debug,
{
    Equal {
        order: usize,
        length: usize,

        #[cfg(debug_assertions)]
        #[cfg_attr(feature = "serde", serde(skip_serializing))]
        text: Option<String>,
    },

    Insert {
        order: usize,
        text: Vec<Token<T>>,
    },

    Delete {
        order: usize,
        deleted_character_count: usize,

        #[cfg(debug_assertions)]
        #[cfg_attr(feature = "serde", serde(skip_serializing))]
        deleted_text: Option<String>,
    },
}

impl<T> Operation<T>
where
    T: PartialEq + Clone + Debug,
{
    /// Creates an equal (retain) operation starting at the given character
    /// offset in the original text
    pub fn create_equal(order: usize, length: usize) -> Self {
        Operation::Equal {
            order,
            length,

            #[cfg(debug_assertions)]
            text: None,
        }
    }

    pub fn create_equal_with_text(order: usize, text: String) -> Self {
        Operation::Equal {
            order,
            length: text.chars().count(),

            #[cfg(debug_assertions)]
            text: Some(text),
        }
    }

    /// Creates an insert operation at the given character offset with the
    /// given tokens
    pub fn create_insert(order: usize, text: Vec<Token<T>>) -> Self {
        Operation::Insert { order, text }
    }

    /// Creates a delete operation at the given character offset for the
    /// specified number of characters
    pub fn create_delete(order: usize, deleted_character_count: usize) -> Self {
        Operation::Delete {
            order,
            deleted_character_count,

            #[cfg(debug_assertions)]
            deleted_text: None,
        }
    }

    pub fn create_delete_with_text(order: usize, text: String) -> Self {
        Operation::Delete {
            order,
            deleted_character_count: text.chars().count(),

            #[cfg(debug_assertions)]
            deleted_text: Some(text),
        }
    }

    fn order(&self) -> usize {
        match self {
            Operation::Equal { order, .. }
            | Operation::Insert { order, .. }
            | Operation::Delete { order, .. } => *order,
        }
    }

    fn type_priority(&self) -> u8 {
        match self {
            Operation::Delete { .. } => 1,
            Operation::Insert { .. } => 2,
            Operation::Equal { .. } => 3,
        }
    }

    /// Compare two operations for processing order during merging. Uses
    /// (order, type, `insertion_index`) with a deterministic content
    /// tiebreaker that avoids allocating.
    pub fn cmp_priority(
        &self,
        self_index: usize,
        other: &Self,
        other_index: usize,
    ) -> std::cmp::Ordering {
        self.order()
            .cmp(&other.order())
            .then_with(|| self.type_priority().cmp(&other.type_priority()))
            .then_with(|| self_index.cmp(&other_index))
            .then_with(|| self.deterministic_content_cmp(other))
    }

    /// Deterministic tiebreaker based on operation content, so that merge
    /// results are identical regardless of which side is left vs right
    fn deterministic_content_cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Operation::Insert { text: t1, .. }, Operation::Insert { text: t2, .. }) => {
                let s1 = t1.iter().flat_map(|t| t.original().chars());
                let s2 = t2.iter().flat_map(|t| t.original().chars());
                s1.cmp(s2)
            }
            (Operation::Equal { length: l1, .. }, Operation::Equal { length: l2, .. }) => {
                l1.cmp(l2)
            }
            (
                Operation::Delete {
                    deleted_character_count: c1,
                    ..
                },
                Operation::Delete {
                    deleted_character_count: c2,
                    ..
                },
            ) => c1.cmp(c2),
            // Different types are already ordered by type_priority
            _ => std::cmp::Ordering::Equal,
        }
    }

    /// Applies the operation to the given `StringBuilder`, returning the
    /// modified `StringBuilder`.
    ///
    /// When compiled in debug mode, panics if a delete operation is attempted
    /// on a range of text that does not match the text to be deleted.
    pub fn apply<'a>(&self, mut builder: StringBuilder<'a>) -> StringBuilder<'a> {
        match self {
            Operation::Equal {
                #[cfg(debug_assertions)]
                text,
                length,
                ..
            } => {
                #[cfg(debug_assertions)]
                debug_assert!(
                    text.as_ref()
                        .is_none_or(|text| builder.get_slice_from_remaining(self.len()) == *text),
                    "Text (`{}`) which is supposed to be equal does not match the text in the \
                     range: `{}`",
                    text.as_ref().unwrap_or(&String::new()),
                    builder.get_slice_from_remaining(self.len())
                );

                builder.retain(*length);
            }
            Operation::Insert { text, .. } => {
                builder.insert(&text.iter().map(Token::original).collect::<String>());
            }
            Operation::Delete {
                #[cfg(debug_assertions)]
                deleted_text,
                deleted_character_count,
                ..
            } => {
                #[cfg(debug_assertions)]
                debug_assert!(
                    deleted_text
                        .as_ref()
                        .is_none_or(|text| builder.get_slice_from_remaining(self.len()) == *text),
                    "Text to-be-deleted `{}` does not match the text in the range: `{}`",
                    deleted_text.as_ref().unwrap_or(&String::new()),
                    builder.get_slice_from_remaining(self.len())
                );

                builder.delete(*deleted_character_count);
            }
        }

        builder
    }

    /// Returns the number of affected characters. May be 0 after
    /// `merge_operations`.
    pub fn len(&self) -> usize {
        match self {
            Operation::Equal { length, .. } => *length,
            Operation::Insert { text, .. } => text.iter().map(Token::get_original_length).sum(),
            Operation::Delete {
                deleted_character_count,
                ..
            } => *deleted_character_count,
        }
    }

    /// Adjusts this operation based on `previous_operation` from the other side
    /// to avoid duplicating or conflicting changes
    #[allow(clippy::too_many_lines)]
    pub fn merge_operations(self, previous_operation: Option<&Self>) -> Operation<T> {
        let operation = self;

        match (operation, previous_operation) {
            (
                Operation::Insert { order, text },
                Some(Operation::Insert {
                    text: previous_inserted_text,
                    ..
                }),
            ) => {
                // In case the current insert's prefix appears in the previously inserted text,
                // we can trim the current insert to only include the non-overlapping part.
                // This way, we don't end up duplicating text.
                let offset_in_tokens =
                    find_longest_prefix_contained_within(previous_inserted_text, &text);

                Operation::create_insert(order, text[offset_in_tokens..].to_vec())
            }

            (
                Operation::Delete {
                    order,
                    deleted_character_count,

                    #[cfg(debug_assertions)]
                    deleted_text,
                },
                Some(Operation::Delete {
                    order: last_delete_order,
                    deleted_character_count: last_delete_deleted_character_count,
                    ..
                }),
            ) => {
                let operation_end_index = order + deleted_character_count;
                let last_delete_end_index =
                    *last_delete_order + *last_delete_deleted_character_count;

                let new_length = deleted_character_count
                    .min(operation_end_index.saturating_sub(last_delete_end_index));

                let overlap = deleted_character_count - new_length;

                #[cfg(debug_assertions)]
                let updated_delete = deleted_text.as_ref().map_or_else(
                    || Operation::create_delete(order + overlap, new_length),
                    |text| {
                        Operation::create_delete_with_text(
                            order + overlap,
                            text.chars()
                                .skip(deleted_character_count - new_length)
                                .collect::<String>(),
                        )
                    },
                );

                #[cfg(not(debug_assertions))]
                let updated_delete = Operation::create_delete(order + overlap, new_length);

                updated_delete
            }

            (
                Operation::Equal {
                    order,
                    length,

                    #[cfg(debug_assertions)]
                    ref text,
                },
                Some(Operation::Delete {
                    order: last_delete_order,
                    deleted_character_count: last_delete_deleted_character_count,
                    ..
                }),
            ) => {
                let last_delete_end_index =
                    *last_delete_order + *last_delete_deleted_character_count;

                let overlap = length.min(last_delete_end_index.saturating_sub(order));

                #[cfg(debug_assertions)]
                let updated_equal = text.as_ref().map_or_else(
                    || Operation::create_equal(order + overlap, length - overlap),
                    |text| {
                        Operation::create_equal_with_text(
                            order + overlap,
                            text.chars().skip(overlap).collect::<String>(),
                        )
                    },
                );

                #[cfg(not(debug_assertions))]
                let updated_equal = Operation::create_equal(order + overlap, length - overlap);

                updated_equal
            }

            (
                ref operation @ Operation::Equal {
                    ref order,
                    #[cfg(debug_assertions)]
                    ref text,
                    ..
                },
                Some(Operation::Equal {
                    order: last_equal_order,
                    length: last_equal_length,
                    #[cfg(debug_assertions)]
                    text: last_equal_text,
                    ..
                }),
            ) => {
                if operation.len() == *last_equal_length && *order == *last_equal_order {
                    // Both sides retained the same span from the original text,
                    // so we deduplicate by zeroing one out. This is safe because
                    // both EditedTexts are derived from the same original, and
                    // matching (order, length) means they cover the same substring
                    #[cfg(debug_assertions)]
                    debug_assert_eq!(
                        text, last_equal_text,
                        "Equal operations with same order and length should have the same text, \
                         but got {operation:?} vs {:?}",
                        Operation::<T>::Equal {
                            order: *last_equal_order,
                            length: *last_equal_length,
                            text: last_equal_text.clone(),
                        },
                    );
                    Operation::create_equal(*order, 0)
                } else {
                    operation.clone()
                }
            }

            (operation, _) => operation,
        }
    }
}

impl<T> Display for Operation<T>
where
    T: PartialEq + Clone + Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Operation::Equal {
                order,
                length,

                #[cfg(debug_assertions)]
                text,
                ..
            } => {
                #[cfg(debug_assertions)]
                {
                    write!(
                        f,
                        "<equal {} from {order}>",
                        text.as_ref()
                            .map(|text| format!("'{}'", text.replace('\n', "\\n")))
                            .unwrap_or(format!("{length} characters")),
                    )
                }

                #[cfg(not(debug_assertions))]
                {
                    write!(f, "<equal {length} from {order}>")
                }
            }
            Operation::Insert { order, text, .. } => {
                write!(
                    f,
                    "<insert '{}' at {order}>",
                    text.iter()
                        .map(Token::original)
                        .collect::<String>()
                        .replace('\n', "\\n"),
                )
            }
            Operation::Delete {
                order,
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
                ..
            } => {
                #[cfg(debug_assertions)]
                {
                    write!(
                        f,
                        "<delete {} from {order}>",
                        deleted_text
                            .as_ref()
                            .map(|text| format!("'{}'", text.replace('\n', "\\n")))
                            .unwrap_or(format!("{deleted_character_count} characters")),
                    )
                }

                #[cfg(not(debug_assertions))]
                {
                    write!(
                        f,
                        "<delete {deleted_character_count} characters from {order}>",
                    )
                }
            }
        }
    }
}

impl<T> Debug for Operation<T>
where
    T: PartialEq + Clone + Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{self}") }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_apply_delete_with_create() {
        let builder = StringBuilder::new("hello world");
        let delete_operation = Operation::<()>::create_delete_with_text(0, "hello ".to_owned());
        let retain_operation = Operation::<()>::create_equal(6, 5);

        let mut builder = delete_operation.apply(builder);
        builder = retain_operation.apply(builder);

        assert_eq!(builder.take(), "world");
    }

    #[test]
    fn test_apply_insert() {
        let builder = StringBuilder::new("hello");

        let retain_operation = Operation::<()>::create_equal(0, 5);
        let insert_operation = Operation::create_insert(5, vec![" my friend".into()]);

        let mut builder = retain_operation.apply(builder);
        builder = insert_operation.apply(builder);

        assert_eq!(builder.take(), "hello my friend");
    }
}
