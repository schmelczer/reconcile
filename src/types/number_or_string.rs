use std::{borrow::Cow, fmt::Debug};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[allow(clippy::cast_precision_loss)]
const INTEGRAL_LIMIT: f64 = (1u64 << 53) as f64;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug, Clone, PartialEq)]
pub enum NumberOrString {
    Number(i64),
    Text(String),
}

#[cfg(feature = "wasm")]
impl TryFrom<JsValue> for NumberOrString {
    type Error = DeserialisationError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        if let Ok(num) = value.clone().try_into() {
            return Ok(NumberOrString::Number(num));
        }

        if let Some(num) = value.clone().as_f64() {
            if num.abs() > INTEGRAL_LIMIT {
                return Err(DeserialisationError::new(
                    "Floating-point number exceeds safe integer limit, use BigInt instead",
                ));
            }

            #[allow(clippy::cast_possible_truncation)]
            return Ok(NumberOrString::Number(num.round() as i64));
        }

        if let Ok(text) = value.try_into() {
            return Ok(NumberOrString::Text(text));
        }

        Err(DeserialisationError::new(
            "Could not parse JsValue as either number or string",
        ))
    }
}

#[cfg(feature = "wasm")]
impl From<NumberOrString> for JsValue {
    fn from(value: NumberOrString) -> Self {
        match value {
            NumberOrString::Number(num) => JsValue::from(num),
            NumberOrString::Text(text) => JsValue::from(text),
        }
    }
}

impl From<i64> for NumberOrString {
    fn from(value: i64) -> Self { NumberOrString::Number(value) }
}

impl From<String> for NumberOrString {
    fn from(value: String) -> Self { NumberOrString::Text(value) }
}

impl From<&str> for NumberOrString {
    fn from(value: &str) -> Self { NumberOrString::Text(value.to_owned()) }
}

impl<'a> From<Cow<'a, str>> for NumberOrString {
    fn from(value: Cow<'a, str>) -> Self { NumberOrString::Text(value.into_owned()) }
}

/// Error type for deserialisation failures
#[cfg(feature = "wasm")]
#[derive(Debug, Clone)]
pub struct DeserialisationError {
    pub message: String,
}

#[cfg(feature = "wasm")]
impl DeserialisationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(feature = "wasm")]
impl std::fmt::Display for DeserialisationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Deserialisation error: {}", self.message)
    }
}

#[cfg(feature = "wasm")]
impl std::error::Error for DeserialisationError {}

#[cfg(feature = "wasm")]
impl From<DeserialisationError> for JsValue {
    fn from(error: DeserialisationError) -> Self { JsValue::from_str(&error.message) }
}
