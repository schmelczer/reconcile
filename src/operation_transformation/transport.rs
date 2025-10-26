use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer, Visitor},
    ser::Serializer,
};

use crate::{CursorPosition, Tokenizer, operation_transformation::Operation};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SimpleOperation {
    Equal { length: usize },

    Insert { text: String },

    Delete { length: usize },
}

impl SimpleOperation {
    pub fn from_operations<T>(operation: &Vec<Operation<T>>) -> Vec<Self>
    where
        T: PartialEq + Clone + Debug,
    {
        let mut result: Vec<Self> = Vec::with_capacity(operation.len());
        let mut previous_equal: Option<usize> = None;

        for operation in operation {
            match operation {
                Operation::Equal { length, .. } => {
                    if let Some(prev_length) = previous_equal {
                        previous_equal = Some(prev_length + *length);
                    } else {
                        previous_equal = Some(*length);
                    }
                }
                Operation::Insert { text, .. } => {
                    if let Some(prev_length) = previous_equal {
                        result.push(SimpleOperation::Equal {
                            length: prev_length,
                        });
                        previous_equal = None;
                    }

                    let text: String = text
                        .iter()
                        .map(super::super::tokenizer::token::Token::original)
                        .collect();
                    result.push(SimpleOperation::Insert { text });
                }
                Operation::Delete {
                    deleted_character_count,
                    ..
                } => {
                    if let Some(prev_length) = previous_equal {
                        result.push(SimpleOperation::Equal {
                            length: prev_length,
                        });
                        previous_equal = None;
                    }

                    result.push(SimpleOperation::Delete {
                        length: *deleted_character_count,
                    });
                }
            }
        }

        if let Some(prev_length) = previous_equal {
            result.push(SimpleOperation::Equal {
                length: prev_length,
            });
        }

        result
    }

    pub fn to_operations<T>(
        simple_operations: Vec<Self>,
        original_text: &str,
        tokenizer: &Tokenizer<T>,
    ) -> Vec<Operation<T>>
    where
        T: PartialEq + Clone + Debug,
    {
        let mut operations: Vec<Operation<T>> = Vec::with_capacity(simple_operations.len());

        let mut order = 0;

        for simple_operation in simple_operations {
            match simple_operation {
                SimpleOperation::Equal { length } => {
                    let original_characters: String =
                        original_text.chars().skip(order).take(length).collect();

                    let original_tokens = tokenizer(&original_characters);
                    for token in original_tokens {
                        operations
                            .push(Operation::create_equal(order, token.get_original_length()));
                        order += token.get_original_length();
                    }
                }
                SimpleOperation::Insert { text } => {
                    let tokens = tokenizer(&text);
                    operations.push(Operation::create_insert(order, tokens));
                }
                SimpleOperation::Delete { length } => {
                    operations.push(Operation::create_delete(order, length));
                    order += length;
                }
            }
        }

        operations
    }
}

#[cfg(feature = "serde")]
impl Serialize for SimpleOperation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // neat idea from https://github.com/spebern/operational-transform-rs/blob/9faa17f0a2b282ac2e09dbb2d29fdaf2ae0bbb4a/operational-transform/src/serde.rs#L14
        match self {
            SimpleOperation::Equal { length } => serializer.serialize_u64(*length as u64),
            SimpleOperation::Insert { text } => serializer.serialize_str(text),
            SimpleOperation::Delete { length } => {
                serializer.serialize_i64(-(i64::try_from(*length).unwrap_or(i64::MAX)))
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SimpleOperation {
    fn deserialize<D>(deserializer: D) -> Result<SimpleOperation, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::fmt;

        struct OperationVisitor;

        impl Visitor<'_> for OperationVisitor {
            type Value = SimpleOperation;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an integer between -2^64 and 2^63 or a string")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SimpleOperation::Equal {
                    length: usize::try_from(value).unwrap_or(usize::MAX),
                })
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SimpleOperation::Delete {
                    length: usize::try_from(-value).unwrap_or(usize::MAX),
                })
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SimpleOperation::Insert {
                    text: value.to_owned(),
                })
            }
        }

        deserializer.deserialize_any(OperationVisitor)
    }
}

/// A serializable representation of the changes made to a text document
/// without the original text.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChangeSet {
    pub operations: Vec<SimpleOperation>,
    pub cursors: Vec<CursorPosition>,
}

impl ChangeSet {
    #[must_use]
    pub fn new(operations: Vec<SimpleOperation>, cursors: Vec<CursorPosition>) -> Self {
        Self {
            operations,
            cursors,
        }
    }
}
