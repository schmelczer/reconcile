use core::fmt::{Debug, Display};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    Side, Token,
    utils::{
        find_longest_prefix_contained_within::find_longest_prefix_contained_within,
        string_builder::StringBuilder,
    },
};

/// Represents a change that can be applied on a `StringBuilder`.
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
        text: Option<String>,
    },

    Insert {
        side: Side,

        order: usize,
        text: Vec<Token<T>>,
    },

    Delete {
        side: Side,

        order: usize,
        deleted_character_count: usize,

        #[cfg(debug_assertions)]
        deleted_text: Option<String>,
    },
}

impl<T> Operation<T>
where
    T: PartialEq + Clone + Debug,
{
    /// Creates an equal operation with the given index.
    /// This operation is used to indicate that the text at the given index
    /// is unchanged.
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

    /// Creates an insert operation with the given index and text.
    pub fn create_insert(order: usize, text: Vec<Token<T>>, side: Side) -> Self {
        Operation::Insert { side, order, text }
    }

    /// Creates a delete operation with the given index and number of
    /// to-be-deleted characters.
    pub fn create_delete(order: usize, deleted_character_count: usize, side: Side) -> Self {
        Operation::Delete {
            side,
            order,
            deleted_character_count,

            #[cfg(debug_assertions)]
            deleted_text: None,
        }
    }

    pub fn create_delete_with_text(order: usize, text: String, side: Side) -> Self {
        Operation::Delete {
            side,
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

    pub fn get_sort_key(&self, insertion_index: usize) -> (usize, usize, usize, String) {
        (
            self.order(),
            match self {
                Operation::Delete { .. } => 1,
                Operation::Insert { .. } => 2,
                Operation::Equal { .. } => 3,
            },
            insertion_index,
            // Make sure that the ordering is deterministic regardless of which text
            // is left or right.
            match self {
                Operation::Equal { length, .. } => length.to_string(),
                Operation::Insert { text, .. } => {
                    text.iter().map(Token::original).collect::<String>()
                }
                Operation::Delete {
                    deleted_character_count,
                    ..
                } => deleted_character_count.to_string(),
            },
        )
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

    /// Returns the number of affected characters. It is always greater than 0
    /// because empty operations cannot be created.
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

    /// Merges the operation with the given context, producing a new operation
    /// and updating the context. This implements a comples FSM that handles
    /// the merging of operations in a way that is consistent with the text.
    /// The contexts are updated in-place.
    #[allow(clippy::too_many_lines)]
    pub fn merge_operations(self, previous_operation: &mut Option<Self>) -> Operation<T> {
        let operation = self;

        match (operation, previous_operation) {
            (
                Operation::Insert { side, order, text },
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

                Operation::create_insert(order, text[offset_in_tokens..].to_vec(), side)
            }

            (
                Operation::Delete {
                    side,
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
                    .min(0.max(operation_end_index as i64 - last_delete_end_index as i64) as usize);

                let overlap = deleted_character_count - new_length;

                #[cfg(debug_assertions)]
                let updated_delete = deleted_text.as_ref().map_or_else(
                    || Operation::create_delete(order + overlap, new_length, side),
                    |text| {
                        Operation::create_delete_with_text(
                            order + overlap,
                            text.chars()
                                .skip(deleted_character_count - new_length)
                                .collect::<String>(),
                            side,
                        )
                    },
                );

                #[cfg(not(debug_assertions))]
                let updated_delete = Operation::create_delete(order + overlap, new_length, side);

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

                let overlap =
                    0.max((length as i64).min(last_delete_end_index as i64 - order as i64));

                #[cfg(debug_assertions)]
                let updated_equal = text.as_ref().map_or_else(
                    || {
                        Operation::create_equal(
                            order + overlap as usize,
                            (length as i64 - overlap) as usize,
                        )
                    },
                    |text| {
                        Operation::create_equal_with_text(
                            order + overlap as usize,
                            text.chars().skip(overlap as usize).collect::<String>(),
                        )
                    },
                );

                #[cfg(not(debug_assertions))]
                let updated_equal = Operation::create_equal(
                    order + overlap as usize,
                    (length as i64 - overlap) as usize,
                );

                updated_equal
            }

            (
                ref operation @ Operation::Equal { ref order, .. },
                Some(Operation::Equal {
                    order: last_equal_order,
                    length: last_equal_length,
                    ..
                }),
            ) => {
                if operation.len() == *last_equal_length && *order == *last_equal_order {
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
                write!(
                    f,
                    "<equal {} from {order}>",
                    text.as_ref()
                        .map(|text| format!("'{}'", text.replace('\n', "\\n")))
                        .unwrap_or(format!("{length} characters")),
                )?;

                #[cfg(not(debug_assertions))]
                write!(f, "<equal {length} from {order}>")?;

                Ok(())
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
                write!(
                    f,
                    "<delete {} from {order}>",
                    deleted_text
                        .as_ref()
                        .map(|text| format!("'{}'", text.replace('\n', "\\n")))
                        .unwrap_or(format!("{deleted_character_count} characters")),
                )?;

                #[cfg(not(debug_assertions))]
                write!(
                    f,
                    "<delete {deleted_character_count} characters from {order}>",
                )?;

                Ok(())
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
        let delete_operation =
            Operation::<()>::create_delete_with_text(0, "hello ".to_owned(), Side::Left);
        let retain_operation = Operation::<()>::create_equal(6, 5);

        let mut builder = delete_operation.apply(builder);
        builder = retain_operation.apply(builder);

        assert_eq!(builder.take(), "world");
    }

    #[test]
    fn test_apply_insert() {
        let builder = StringBuilder::new("hello");

        let retain_operation = Operation::<()>::create_equal(0, 5);
        let insert_operation = Operation::create_insert(5, vec![" my friend".into()], Side::Right);

        let mut builder = retain_operation.apply(builder);
        builder = insert_operation.apply(builder);

        assert_eq!(builder.take(), "hello my friend");
    }
}
