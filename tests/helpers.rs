//------------------------------------------------------------------------------
// Copyright (c) 2025                 orgrinrt           orgrinrt@ikiuni.dev
//                                    Hiisi Digital Oy   contact@hiisi.digital
//------------------------------------------------------------------------------

//! The helpers here are used from inside other crates' proc macros, so these exercise them the
//! way a caller does: build the input, run the helper, and compare the tokens it produces.

use ohelpers_proc_macros::{quote_if, token_name, TokenStream2};
use quote::quote;

/// Tokens compare by their printed form, since `TokenStream` has no equality of its own.
fn rendered(tokens: TokenStream2) -> String {
    tokens.to_string()
}

#[test]
fn quote_if_emits_the_block_when_the_predicate_holds() {
    let emitted: TokenStream2 = quote_if!(true, {
        let x = 1;
    });
    assert_eq!(rendered(emitted), rendered(quote! { let x = 1; }));
}

#[test]
fn quote_if_emits_nothing_when_the_predicate_does_not_hold() {
    let emitted: TokenStream2 = quote_if!(false, {
        let x = 1;
    });
    assert!(rendered(emitted).is_empty(), "nothing is emitted");
}

#[test]
fn quote_if_interpolates_an_ident_form() {
    let inner = quote! { some_call() };
    let emitted: TokenStream2 = quote_if!(true, inner);
    assert_eq!(rendered(emitted), rendered(quote! { some_call() }));
}

#[test]
fn quote_if_some_follows_the_option() {
    let present = Some(quote! { value });
    let emitted: TokenStream2 = quote_if!(some present);
    assert_eq!(rendered(emitted), rendered(quote! { value }));

    let absent: Option<TokenStream2> = None;
    let emitted: TokenStream2 = quote_if!(some absent);
    assert!(
        rendered(emitted).is_empty(),
        "an absent option emits nothing"
    );
}

#[test]
fn token_name_gives_the_last_segment_of_a_type_path() {
    struct Widget;
    let name = token_name!(ty Widget);
    assert_eq!(name, "Widget", "the path is trimmed to its last segment");
}

#[test]
fn token_name_names_a_generic_parameter_by_its_concrete_type() {
    fn named<T>() -> &'static str {
        token_name!(ty T)
    }
    assert_eq!(named::<u32>(), "u32");
    assert_eq!(named::<String>(), "String");
}
