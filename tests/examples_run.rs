//! Every example runs, and prints what its own prose says it does.
//!
//! An example that does not run is worse than no example: a reader copies it, it fails, and the
//! failure reads as their mistake. So each is executed rather than only compiled, and the
//! assertions are on what it printed.

use std::process::Command;

/// Runs one example and returns its output.
fn run(name: &str) -> String {
    let output = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "--example", name])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "{name} failed:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn every_example_in_the_directory_is_covered_here() {
    // Without this, adding an example and forgetting to test it is invisible: the tests below
    // all still pass and the new file is never run.
    let covered = ["every_helper", "one_helper"];
    let mut present: Vec<String> =
        std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/examples"))
            .expect("the examples directory")
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
            .map(|e| e.path().file_stem().unwrap().to_string_lossy().into_owned())
            .collect();
    present.sort();
    assert_eq!(present, covered);
}

#[test]
fn one_helper_shows_the_fragment_appearing_and_not_appearing() {
    let out = run("one_helper");
    assert!(out.contains("with the condition true:  # [derive (Debug)]"));
    // The empty case is the half worth asserting: `quote_if!` emits nothing rather than an
    // empty block, and the two are different once the result is interpolated.
    assert!(out.contains(r##"with it false:            """##));
    assert!(out.contains("some(Some(..)):           value"));
    assert!(out.contains(r##"some(None):               """##));
}

#[test]
fn every_helper_covers_each_one() {
    let out = run("every_helper");
    assert!(out.contains("a concrete type:   Widget"));
    // The case `token_name!` exists for: inside a generic the source says `T`, and this is
    // what `T` was instantiated with.
    assert!(out.contains("through a generic: u32"));
    assert!(out.contains("and another:       String"));
    assert!(out.contains(r##"true  -> "# [derive (Debug)]""##));
    assert!(out.contains(r##"false -> """##));
}
