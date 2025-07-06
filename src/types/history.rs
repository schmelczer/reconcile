#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "wasm")]
pub enum History {
    Unchanged = "Unchanged",
    AddedFromLeft = "AddedFromLeft",
    AddedFromRight = "AddedFromRight",
    RemovedFromLeft = "RemovedFromLeft",
    RemovedFromRight = "RemovedFromRight",
}

/// Simple enum for describing the result of `reconcile` in a flat list.
/// When compiled to WASM, the enum values are the same as their names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(not(feature = "wasm"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum History {
    Unchanged,
    AddedFromLeft,
    AddedFromRight,
    RemovedFromLeft,
    RemovedFromRight,
}
