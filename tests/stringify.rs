//------------------------------------------------------------------------------
// Copyright (c) 2025                 orgrinrt           orgrinrt@ikiuni.dev
//                                    Hiisi Digital Oy   contact@hiisi.digital
//------------------------------------------------------------------------------

//! Every arm of `token_name!`, and the wrapper the `parsable` arms are built on.
//!
//! The `parsable` arms reach `StringableParsable`'s `Display`, which asks
//! `parsable_as_string` for the tokens, which asks `ToTokens`. That impl used to read
//! `(*self).to_tokens(tokens)`, and dereferencing a `&StringableParsable<T>` gives a
//! `StringableParsable<T>`, so it called itself until the stack ran out. Every arm below
//! that goes through `parsable` therefore aborted the process rather than failing, which is
//! why none of them had a test: a suite cannot report on a test that takes the runner with
//! it.

use ohelpers_proc_macros::stringify::{parsable_as_string, peekable_as_string, StringableParsable};
use ohelpers_proc_macros::token_name;
use quote::{quote, ToTokens};
use syn::Token;

#[test]
fn stringable_parsable_yields_the_tokens_it_wraps() {
    let wrapped = StringableParsable::from(quote! { a + b });
    assert_eq!(wrapped.to_token_stream().to_string(), "a + b");
}

#[test]
fn stringable_parsable_displays_the_tokens_it_wraps() {
    // The regression test for the recursion. Before the fix this did not fail, it
    // overflowed the stack and killed the test binary.
    let wrapped = StringableParsable::from(quote! { a + b });
    assert_eq!(wrapped.to_string(), "a + b");
}

#[test]
fn stringable_parsable_wraps_a_boxed_trait_object() {
    // The shape `token_name!(parsable ...)` builds: the default type parameter, reached
    // through the one blanket `From`.
    let wrapped: StringableParsable = StringableParsable::from(
        Box::new(quote! { boxed }) as Box<dyn ToTokens>,
    );
    assert_eq!(wrapped.to_string(), "boxed");
}

#[test]
fn parsable_as_string_renders_any_tokens() {
    assert_eq!(parsable_as_string(quote! { x . y }), "x . y");
    assert_eq!(parsable_as_string(quote! {}), "");
}

#[test]
fn peekable_as_string_names_the_token_a_peek_stands_for() {
    assert_eq!(peekable_as_string(Token![;]), "`;`");
    assert_eq!(peekable_as_string(Token![->]), "`->`");
}

// --- every arm of token_name! ------------------------------------------------------------

#[test]
fn arm_ty_gives_the_last_path_segment() {
    struct Widget;
    assert_eq!(token_name!(ty Widget), "Widget");
    assert_eq!(token_name!(ty std::string::String), "String");
    assert_eq!(token_name!(ty u32), "u32", "a primitive has one segment");
}

#[test]
fn arm_peekable_ty_names_the_token_type() {
    assert_eq!(token_name!(peekable ty Token![;]), "`;`");
}

#[test]
fn arm_peekable_names_the_token_a_value_peeks_for() {
    assert_eq!(token_name!(peekable Token![;]), "`;`");
}

#[test]
fn arm_parsable_renders_a_value_that_carries_tokens() {
    let tokens = quote! { some_call () };
    assert_eq!(token_name!(parsable tokens), "some_call ()");
}

#[test]
fn arm_parsable_ty_trims_to_the_last_path_segment() {
    let path = quote! { std::string::String };
    assert_eq!(
        token_name!(parsable ty path),
        "String",
        "the rendered path is trimmed the way the ty arm trims a type name"
    );
}

#[test]
fn arm_parsable_ty_passes_through_an_unqualified_name() {
    let bare = quote! { Widget };
    assert_eq!(token_name!(parsable ty bare), "Widget");
}

#[test]
fn arm_ident_names_the_identifier() {
    let ident = proc_macro2::Ident::new("counter", proc_macro2::Span::call_site());
    assert_eq!(token_name!(ident ident), "counter");
}

#[test]
fn arm_display_uses_the_display_impl() {
    assert_eq!(token_name!(display 42), "42");
    assert_eq!(token_name!(display "text"), "text");
}
