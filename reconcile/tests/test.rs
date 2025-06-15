mod example_document;

use std::{fs, path::Path};

use example_document::ExampleDocument;
use reconcile::{reconcile, reconcile_with_cursors};
use serde::Deserialize;

#[test]
fn test_document_one_way_without_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq_without_cursors(&reconcile(
            &doc.parent(),
            &doc.left().text,
            &doc.right().text,
        ));
    }
}

#[test]
fn test_document_one_way_with_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq(&reconcile_with_cursors(
            &doc.parent(),
            doc.left(),
            doc.right(),
        ));
    }
}

#[test]
fn test_document_inverse_way_without_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq_without_cursors(&reconcile(
            &doc.parent(),
            &doc.right().text,
            &doc.left().text,
        ));
    }
}

#[test]
fn test_document_inverse_way_with_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq(&reconcile_with_cursors(
            &doc.parent(),
            doc.right(),
            doc.left(),
        ));
    }
}

fn get_all_documents() -> Vec<ExampleDocument> {
    let examples_dir = Path::new("tests/examples");
    let entries = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .collect::<Vec<_>>();

    let mut documents = Vec::new();

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
            let file = fs::File::open(&path).expect("Failed to open example file");
            for document in serde_yaml::Deserializer::from_reader(file) {
                let doc =
                    ExampleDocument::deserialize(document).expect("Failed to deserialize document");
                documents.push(doc);
            }
        }
    }

    documents
}
