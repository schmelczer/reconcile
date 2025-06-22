use core::fmt::{Debug, Display};
use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    Token,
    operation_transformation::ordered_operation::OrderedOperation,
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
    T: PartialEq + Clone + std::fmt::Debug,
{
    Equal {
        length: usize,

        #[cfg(debug_assertions)]
        text: Option<String>,
    },

    Insert {
        text: Vec<Token<T>>,
    },

    Delete {
        deleted_character_count: usize,

        #[cfg(debug_assertions)]
        deleted_text: Option<String>,
    },
}

impl<T> Operation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    /// Creates an equal operation with the given index.
    /// This operation is used to indicate that the text at the given index
    /// is unchanged.
    pub fn create_equal(length: usize) -> Self {
        Operation::Equal {
            length,

            #[cfg(debug_assertions)]
            text: None,
        }
    }

    pub fn create_equal_with_text(text: String) -> Self {
        Operation::Equal {
            length: text.chars().count(),

            #[cfg(debug_assertions)]
            text: Some(text),
        }
    }

    /// Creates an insert operation with the given index and text.
    pub fn create_insert(text: Vec<Token<T>>) -> Self { Operation::Insert { text } }

    /// Creates a delete operation with the given index and number of
    /// to-be-deleted characters.
    pub fn create_delete(deleted_character_count: usize) -> Self {
        Operation::Delete {
            deleted_character_count,

            #[cfg(debug_assertions)]
            deleted_text: None,
        }
    }

    pub fn create_delete_with_text(text: String) -> Self {
        Operation::Delete {
            deleted_character_count: text.chars().count(),

            #[cfg(debug_assertions)]
            deleted_text: Some(text),
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
                    text.as_ref().unwrap_or(&"".to_owned()),
                    builder.get_slice_from_remaining(self.len())
                );

                builder.retain(*length)
            }
            Operation::Insert { text, .. } => {
                builder.insert(&text.iter().map(Token::original).collect::<String>())
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
                    deleted_text.as_ref().unwrap_or(&"".to_owned()),
                    builder.get_slice_from_remaining(self.len())
                );

                builder.delete(*deleted_character_count)
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
    pub fn merge_operations_with_context(
        self,
        order: usize,
        previous_operation: &mut Option<OrderedOperation<T>>,
    ) -> OrderedOperation<T> {
        println!("mergin: {self} (order {order}) - previous: {previous_operation:?}");
        let operation = self;

        match (operation, previous_operation) {
            (
                Operation::Insert { text },
                Some(OrderedOperation {
                    operation:
                        Operation::Insert {
                            text: previous_inserted_text,
                            ..
                        },
                    ..
                }),
            ) => {
                // In case the current insert's prefix appears in the previously inserted text,
                // we can trim the current insert to only include the non-overlapping part.
                // This way, we don't end up duplicating text.
                let offset_in_tokens =
                    find_longest_prefix_contained_within(previous_inserted_text, &text);

                let trimmed_operation = Operation::create_insert(text[offset_in_tokens..].to_vec());

                OrderedOperation {
                    order,
                    operation: trimmed_operation,
                }
            }

            (
                Operation::Delete {
                    #[cfg(debug_assertions)]
                    deleted_text,
                    deleted_character_count,
                },
                Some(
                    last_delete @ OrderedOperation {
                        operation: Operation::Delete { .. },
                        ..
                    },
                ),
            ) => {
                let operation_end_index = order + deleted_character_count;
                let last_delete_end_index = last_delete.order + last_delete.operation.len();

                let new_length = deleted_character_count
                    .min(0.max(operation_end_index as i64 - last_delete_end_index as i64) as usize);

                let overlap = deleted_character_count - new_length;

                #[cfg(debug_assertions)]
                let updated_delete = deleted_text.as_ref().map_or_else(
                    || Operation::create_delete(new_length),
                    |text| {
                        Operation::create_delete_with_text(
                            text.chars()
                                .skip((deleted_character_count - new_length) as usize)
                                .collect::<String>(),
                        )
                    },
                );

                #[cfg(not(debug_assertions))]
                let updated_delete = Operation::create_delete(new_length);

                OrderedOperation {
                    order: order + overlap,
                    operation: updated_delete,
                }
            }

            (
                ref operation @ Operation::Equal {
                    length,
                    #[cfg(debug_assertions)]
                    ref text,
                    ..
                },
                Some(
                    last_delete @ OrderedOperation {
                        operation: Operation::Delete { .. },
                        ..
                    },
                ),
            ) => {
                let last_delete_end_index = last_delete.order + last_delete.operation.len();

                let overlap =
                    0.max((length as i64).min(last_delete_end_index as i64 - order as i64));

                #[cfg(debug_assertions)]
                let updated_equal = text.as_ref().map_or_else(
                    || Operation::create_equal((length as i64 - overlap) as usize),
                    |text| {
                        Operation::create_equal_with_text(
                            text.chars().skip(overlap as usize).collect::<String>(),
                        )
                    },
                );

                #[cfg(not(debug_assertions))]
                let updated_equal = Operation::create_equal((length as i64 - overlap) as usize);

                OrderedOperation {
                    order: order + overlap as usize,
                    operation: updated_equal,
                }
            }

            (
                operation @ Operation::Equal { .. },
                Some(
                    last_equal @ OrderedOperation {
                        operation: Operation::Equal { .. },
                        ..
                    },
                ),
            ) => OrderedOperation {
                order,
                operation: if operation.len() == last_equal.operation.len()
                    && order == last_equal.order
                {
                    Operation::create_equal(0)
                } else {
                    operation
                },
            },

            (operation, _) => OrderedOperation { order, operation },
        }
    }
}

impl<T> Display for Operation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Operation::Equal {
                length,

                #[cfg(debug_assertions)]
                text,
            } => {
                #[cfg(debug_assertions)]
                write!(
                    f,
                    "<equal {}>",
                    text.as_ref()
                        .map(|text| format!("'{}'", text.replace('\n', "\\n")))
                        .unwrap_or(format!("{length} characters")),
                )?;

                #[cfg(not(debug_assertions))]
                write!(f, "<equal {length}>")?;

                Ok(())
            }
            Operation::Insert { text } => {
                write!(
                    f,
                    "<insert '{}'>",
                    text.iter()
                        .map(Token::original)
                        .collect::<String>()
                        .replace('\n', "\\n"),
                )
            }
            Operation::Delete {
                deleted_character_count,

                #[cfg(debug_assertions)]
                deleted_text,
            } => {
                #[cfg(debug_assertions)]
                write!(
                    f,
                    "<delete {}>",
                    deleted_text
                        .as_ref()
                        .map(|text| format!("'{}'", text.replace('\n', "\\n")))
                        .unwrap_or(format!("{deleted_character_count} characters")),
                )?;

                #[cfg(not(debug_assertions))]
                write!(f, "<delete {deleted_character_count} characters>",)?;

                Ok(())
            }
        }
    }
}

impl<T> Debug for Operation<T>
where
    T: PartialEq + Clone + std::fmt::Debug,
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
        let delete_operation = Operation::<()>::create_delete_with_text("hello ".to_owned());
        let retain_operation = Operation::<()>::create_equal(5);

        let mut builder = delete_operation.apply(builder);
        builder = retain_operation.apply(builder);

        assert_eq!(builder.build(), "world");
    }

    #[test]
    fn test_apply_insert() {
        let builder = StringBuilder::new("hello");

        let retain_operation = Operation::<()>::create_equal(5);
        let insert_operation = Operation::create_insert(vec![" my friend".into()]);

        let mut builder = retain_operation.apply(builder);
        builder = insert_operation.apply(builder);

        assert_eq!(builder.build(), "hello my friend");
    }
}
