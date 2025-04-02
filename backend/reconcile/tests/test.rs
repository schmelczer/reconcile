mod example_document;
use std::{fs, path::Path};

use example_document::ExampleDocument;
use reconcile::{reconcile, reconcile_with_cursors};

#[test]
fn test_with_examples() {
    let examples_dir = Path::new("tests/examples");
    let mut entries = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| {
        let path = entry
            .as_ref()
            .expect("Failed to read directory entry")
            .path();
        path.file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.split('.').next().unwrap().parse::<i32>().ok())
            .unwrap_or_default()
    });

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
            let doc = ExampleDocument::from_yaml(&path);
            println!("Testing with example from {}", path.display());

            doc.assert_eq_without_cursors(&reconcile(
                &doc.parent(),
                &doc.left().text,
                &doc.right().text,
            ));

            doc.assert_eq(&reconcile_with_cursors(
                &doc.parent(),
                doc.left(),
                doc.right(),
            ));
        }
    }
}
