use std::panic;

use diff_match_patch_rs::{Compat, DiffMatchPatch, PatchInput};
use reconcile_text::{BuiltinTokenizer, reconcile};

fn dmp_merge(parent: &str, left: &str, right: &str) -> Option<(String, Vec<bool>)> {
    let parent = parent.to_owned();
    let left = left.to_owned();
    let right = right.to_owned();

    // diff-match-patch-rs can panic on some inputs, so we catch that.
    panic::catch_unwind(|| {
        let dmp = DiffMatchPatch::new();
        let diffs = dmp.diff_main::<Compat>(&parent, &left).ok()?;
        let patches = dmp
            .patch_make(PatchInput::new_text_diffs(&parent, &diffs))
            .ok()?;
        dmp.patch_apply(&patches, &right).ok()
    })
    .ok()
    .flatten()
}

fn try_merge(parent: &str, left: &str, right: &str) {
    let dmp_result = dmp_merge(parent, left, right);

    let reconcile_result = reconcile(
        parent,
        &left.into(),
        &right.into(),
        &*BuiltinTokenizer::Word,
    )
    .apply()
    .text();

    println!("Parent: {parent:?}");
    println!("Left:   {left:?}");
    println!("Right:  {right:?}");
    println!();
    match dmp_result {
        Some((result, applied)) => {
            println!("diff-match-patch: {result:?}");
            println!("Patch success:   {applied:?}");
        }
        None => println!("diff-match-patch: <panic or error>"),
    }
    println!("reconcile-text:   {reconcile_result:?}");
    println!();
}

/// Compares applying parent → left patches to right with three-way merging.
/// Shows both a garbled result reported as successful and a reported patch failure.
///
/// Run it with:
/// `cargo run --example compare-with-diff-match-patch`
fn main() {
    // Example 1
    // Two users edit the same short phrase. Alice replaces "old(!) broken"
    // with "new improved", deleting "broken", while Bob replaces "broken"
    // with "working". These changes overlap.
    //
    // This comparison diffs parent → left and applies the patch to right.
    // diff-match-patch produces garbled text ("impovind") but reports
    // patch success as [true].
    //
    // reconcile-text resolves the overlap to "new improved working code".

    println!("── Example 1: overlapping edits ──");
    try_merge(
        "old(!) broken code",
        "new improved code",
        "old(!) working code",
    );

    // Example 2
    // Alice adds a sentence in the middle of a paragraph. Bob rephrases
    // the same paragraph. Because the patch context from Alice's edit no
    // longer matches closely enough in Bob's version, diff-match-patch
    // leaves Bob's text unchanged and reports patch success as [false].
    // The missing sentence is a reported patch failure.
    //
    // reconcile-text retains Alice's sentence alongside Bob's rewrite.

    println!("── Example 2: reported patch failure ──");
    try_merge(
        "We used the existing parsing approach for processing. The output was saved to the \
         database.",
        "We used the existing parsing approach for processing. Always validate the schema! The \
         output was saved to the database.",
        "We adopted a brand new analysis pipeline for execution. The results were written to \
         cloud storage.",
    );
}
