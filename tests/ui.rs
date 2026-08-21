//! What the crate refuses, and the words it refuses in.
//!
//! A recorded stderr is the diagnostic a caller sees, so a change to the wording shows up
//! as a diff rather than silently.

use std::fs;

/// How many cases this suite expects to find.
///
/// `trybuild` is given a glob and a glob matching nothing is not an error, so without this
/// the suite would pass having checked no case at all. Counted from the directory rather
/// than from a number this file also writes.
const EXPECTED_CASES: usize = 1;

#[test]
fn the_refusals_still_refuse() {
    let found = fs::read_dir("tests/ui")
        .expect("the compile-fail case directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .count();
    assert_eq!(
        found, EXPECTED_CASES,
        "expected {EXPECTED_CASES} compile-fail cases in tests/ui, found {found}. \
         Adding one means raising the constant; losing one means something deleted it."
    );

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
