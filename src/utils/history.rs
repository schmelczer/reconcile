#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(feature = "wasm")]
pub enum History {
    Unchanged = "Unchanged",
    AddedFromLeft = "AddedFromLeft",
    AddedFromRight = "AddedFromRight",
    RemovedFromLeft = "RemovedFromLeft",
    RemovedFromRight = "RemovedFromRight",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(not(feature = "wasm"))]
pub enum History {
    Unchanged,
    AddedFromLeft,
    AddedFromRight,
    RemovedFromLeft,
    RemovedFromRight,
}
