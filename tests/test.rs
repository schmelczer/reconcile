mod example_document;

use std::{fs, path::Path};

use example_document::ExampleDocument;
use reconcile_text::{BuiltinTokenizer, EditedText, reconcile};
use serde::Deserialize;

#[test]
fn test_document_one_way_without_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq_without_cursors(
            &reconcile(
                &doc.parent(),
                &doc.left().text().into(),
                &doc.right().text().into(),
                &*BuiltinTokenizer::Word,
            )
            .apply()
            .text(),
        );
    }
}

#[test]
fn test_document_one_way_with_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq(&reconcile(
            &doc.parent(),
            &doc.left(),
            &doc.right(),
            &*BuiltinTokenizer::Word,
        ));
    }
}

#[test]
fn test_document_one_way_with_cursors_and_serialisation() {
    for doc in &get_all_documents() {
        let parent = doc.parent();
        let left_operations =
            EditedText::from_strings_with_tokenizer(&parent, &doc.left(), &*BuiltinTokenizer::Word);
        let right_operations = EditedText::from_strings_with_tokenizer(
            &parent,
            &doc.right(),
            &*BuiltinTokenizer::Word,
        );

        let serialised_left = serde_yaml::from_str(
            &serde_yaml::to_string(&left_operations.serialise_as_change_set()).unwrap(),
        )
        .unwrap();
        let serialised_right = serde_yaml::from_str(
            &serde_yaml::to_string(&right_operations.serialise_as_change_set()).unwrap(),
        )
        .unwrap();

        let restored_left_operations =
            EditedText::from_change_set(&parent, serialised_left, &*BuiltinTokenizer::Word);
        let restored_right_operations =
            EditedText::from_change_set(&parent, serialised_right, &*BuiltinTokenizer::Word);

        doc.assert_eq(&restored_left_operations.merge(restored_right_operations));
    }
}

#[test]
fn test_document_inverse_way_without_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq_without_cursors(
            &reconcile(
                &doc.parent(),
                &doc.right().text().into(),
                &doc.left().text().into(),
                &*BuiltinTokenizer::Word,
            )
            .apply()
            .text(),
        );
    }
}

#[test]
fn test_document_inverse_way_with_cursors() {
    for doc in &get_all_documents() {
        doc.assert_eq(&reconcile(
            &doc.parent(),
            &doc.right(),
            &doc.left(),
            &*BuiltinTokenizer::Word,
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
