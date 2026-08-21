//! The exported macros resolve in a consumer that is not shaped like this crate.
//!
//! Everything these macros emit lands somewhere else, so nothing about it can be checked
//! from in here: this crate has `quote` under that name, has `std`, and has the prelude, so
//! every spelling works locally whether or not it is the right one.
//!
//! `token_name!(parsable ..)` had three names in it that only resolved under those
//! conditions: a bare `quote::ToTokens`, a `Box::new` through the prelude, and a
//! `.to_string()` through the prelude. The neighbouring arm in the same macro was already
//! written with absolute paths, which is what made the difference easy to read past.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Builds a throwaway crate against this one, with a manifest of its own.
///
/// `deps` is pasted into the consumer's `[dependencies]`, so a test can rename quote or
/// leave it out entirely.
fn consumer_compiles(
    name: &str,
    features: &str,
    deps: &str,
    attrs: &str,
    body: &str,
) -> (bool, String) {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/target/consumers")).join(name);
    fs::create_dir_all(root.join("src")).expect("the consumer directory");

    let features_list = if features.is_empty() {
        String::new()
    } else {
        features
            .split(',')
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ")
    };

    fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.0.0"
edition = "2021"

[dependencies.ohelpers_proc_macros]
path = "{crate_dir}"
default-features = false
features = [{features_list}]

[dependencies]
{deps}

[workspace]
"#,
            crate_dir = env!("CARGO_MANIFEST_DIR"),
        ),
    )
    .expect("the consumer manifest");

    fs::write(
        root.join("src").join("lib.rs"),
        format!("{attrs}\n{body}\n"),
    )
    .expect("the consumer source");

    let output = Command::new(env!("CARGO"))
        .args(["check", "--quiet"])
        .current_dir(&root)
        .env(
            "CARGO_TARGET_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/target/consumers/target"),
        )
        .output()
        .expect("cargo runs");

    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// A use of the arm whose paths were wrong.
const USES_PARSABLE: &str = r#"
pub fn name_of(tokens: &proc_macro2::TokenStream) -> alloc::string::String {
    ohelpers_proc_macros::token_name!(parsable tokens)
}
"#;

/// The same, for a consumer that has `std` and so has `String` in the prelude.
const USES_PARSABLE_STD: &str = r#"
pub fn name_of(tokens: &proc_macro2::TokenStream) -> String {
    ohelpers_proc_macros::token_name!(parsable tokens)
}
"#;

#[test]
fn a_plain_consumer_can_use_it() {
    let (ok, err) = consumer_compiles(
        "plain",
        "",
        "quote = \"1.0\"\nproc-macro2 = \"1.0\"",
        "",
        USES_PARSABLE_STD,
    );
    assert!(
        ok,
        "an ordinary consumer can use `token_name!(parsable ..)`:\n{err}"
    );
}

#[test]
fn a_no_std_consumer_can_use_it_without_naming_alloc() {
    // The macro reaches `Box` and `ToString` through re-exports from this crate rather than
    // through `std`, `alloc` or the prelude, so the consumer needs `extern crate alloc` only
    // for its own `String` in the signature, not for anything the macro emits.
    let (ok, err) = consumer_compiles(
        "no_std",
        "no_alloc",
        "quote = { version = \"1.0\", default-features = false }\nproc-macro2 = \"1.0\"",
        "#![no_std]\nextern crate alloc;",
        USES_PARSABLE,
    );
    assert!(
        ok,
        "a `#![no_std]` consumer can use `token_name!(parsable ..)`:\n{err}"
    );
}

#[test]
fn a_consumer_that_renamed_quote_can_use_it() {
    // The bare `quote::ToTokens` resolved only where the dependency was called `quote`. A
    // crate that renamed it got `could not find 'quote' in the list of imported crates`,
    // spanned inside the macro, which is the same failure derive_display was carrying and
    // the reason every path the macros emit is absolute.
    let (ok, err) = consumer_compiles(
        "renamed_quote",
        "",
        "quoting = { package = \"quote\", version = \"1.0\" }\nproc-macro2 = \"1.0\"",
        "",
        USES_PARSABLE_STD,
    );
    assert!(ok, "a consumer that renamed quote can use it:\n{err}");
}

#[test]
fn a_consumer_with_a_type_named_box_is_unaffected() {
    // `Box::new` through the prelude is shadowed by anything the consumer calls `Box`, and
    // the macro would then call a constructor that is not the one it meant. The absolute
    // path cannot be shadowed.
    let body = format!(
        r#"
pub struct Box;
impl Box {{ pub fn new<T>(_: T) -> Self {{ Box }} }}
{USES_PARSABLE_STD}
"#
    );
    let (ok, err) = consumer_compiles(
        "shadowed_box",
        "",
        "quote = \"1.0\"\nproc-macro2 = \"1.0\"",
        "",
        &body,
    );
    assert!(ok, "a consumer with its own `Box` is unaffected:\n{err}");
}
