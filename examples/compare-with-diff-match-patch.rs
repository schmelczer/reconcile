use std::panic;

use diff_match_patch_rs::{Compat, DiffMatchPatch, PatchInput};
use reconcile_text::{BuiltinTokenizer, reconcile};

fn dmp_merge(parent: &str, left: &str, right: &str) -> Option<String> {
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
        let (result, _) = dmp.patch_apply(&patches, &right).ok()?;
        Some(result)
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
        Some(r) => println!("diff-match-patch: {r:?}"),
        None => println!("diff-match-patch: <panic or error>"),
    }
    println!("reconcile-text:   {reconcile_result:?}");
    println!();
}

/// Demonstrates cases where diff-match-patch silently produces incorrect
/// output, while reconcile-text preserves both users' edits correctly
///
/// Run it with:
/// `cargo run --example compare-with-diff-match-patch`
fn main() {
    // Example 1
    // Two users edit the same short phrase. Alice replaces "old(!)" with
    // "new improved", Bob replaces "broken" with "working". These are
    // independent changes to adjacent words.
    //
    // diff-match-patch has no common ancestor, so it diffs parent → left
    // and applies the patch to right. The character-level patches overlap
    // and produce garbled text ("impovind"). It reports success.
    //
    // reconcile-text sees both changes relative to the parent and merges
    // them cleanly.

    println!("── Example 1: adjacent edits ──");
    try_merge(
        "old(!) broken code",
        "new improved code",
        "old(!) working code",
    );

    // Example 2
    // Alice adds a sentence. Bob rewrites the surrounding text. Because
    // diff-match-patch works without a common ancestor, Alice's entire
    // sentence is silently lost.

    println!("── Example 2: sentence lost ──");
    // Alice adds a sentence in the middle of a paragraph. Bob rephrases
    // the same paragraph. Because the patch context from Alice's edit no
    // longer appears in Bob's version, diff-match-patch silently drops
    // Alice's entire sentence.
    //
    // reconcile-text understands both edits relative to the common ancestor
    // and keeps both.
    try_merge(
        "We used the existing parsing approach for processing. The output was saved to the database.",
        "We used the existing parsing approach for processing. Always validate the schema! The output was saved to the database.",
        "We adopted a brand new analysis pipeline for execution. The results were written to cloud storage.",
    );
}
