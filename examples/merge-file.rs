use std::{env, fs, process};

use reconcile_merge::{BuiltinTokenizer, reconcile};

/// Merges three versions of a file: mine, base, and theirs.
/// Implement a trivial version git merge-file (https://git-scm.com/docs/git-merge-file)
///
/// Run it with:
/// `cargo run --example merge-file my.txt base.txt their.txt [output_file.txt]`
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 || args.len() > 5 {
        eprintln!("Usage: merge-file <mine> <base> <theirs> [output]");
        process::exit(1);
    }

    let mine_file = &args[1];
    let base_file = &args[2];
    let theirs_file = &args[3];
    let output_file = args.get(4);

    // Read files
    let mine_content = fs::read_to_string(mine_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", mine_file, e);
        process::exit(1);
    });

    let base_content = fs::read_to_string(base_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", base_file, e);
        process::exit(1);
    });

    let theirs_content = fs::read_to_string(theirs_file).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", theirs_file, e);
        process::exit(1);
    });

    // Perform the merge using reconcile
    let result = reconcile(
        &base_content,
        &mine_content.into(),
        &theirs_content.into(),
        &*BuiltinTokenizer::Word,
    );

    let merged_content = result.apply().text();

    // Write the result
    if let Some(output_path) = output_file {
        if let Err(e) = fs::write(output_path, merged_content) {
            eprintln!("Error writing to {}: {}", output_path, e);
            process::exit(1);
        }
    } else {
        print!("{}", merged_content);
    }
}
