#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum History {
    Unchanged = "Unchanged",
    AddedFromLeft = "AddedFromLeft",
    AddedFromRight = "AddedFromRight",
    RemovedFromLeft = "RemovedFromLeft",
    RemovedFromRight = "RemovedFromRight",
}
