mod operation;
mod operation_sequence;

pub use operation::Operation;
pub use operation_sequence::OperationSequence;

#[cfg(test)]
mod test {

    #[test]
    fn test_merge() {
        // let mut original = Rope::from_str("hello world!");
        // let edit_1 = "hi, world";
        // let edit_2 = "hello, my friend!";

        // let mut operations_1 = calculate_operations(&original.to_string(), edit_1, 1.0).unwrap();
        // let mut operations_2 = calculate_operations(&original.to_string(), edit_2, 1.0).unwrap();

        // let result =
        //     merge_and_apply_operations(&mut original, &mut operations_1, &mut operations_2)
        //         .unwrap();

        // assert_eq!(result, "hey, my friend!");
    }
}
