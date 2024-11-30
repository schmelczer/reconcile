#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate reconcile;

fuzz_target!(|texts: (String, String, String)| {
    let (original, left, right) = texts;
    reconcile::reconcile(&original, &left, &right);
});
