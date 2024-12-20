#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|texts: (String, String, String)| {
    let (original, left, right) = texts;
    let _ = reconcile::reconcile(&original, &left, &right);
});
