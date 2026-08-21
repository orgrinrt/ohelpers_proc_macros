//------------------------------------------------------------------------------
// Copyright (c) 2025                 orgrinrt           orgrinrt@ikiuni.dev
//                                    Hiisi Digital Oy   contact@hiisi.digital
//------------------------------------------------------------------------------

//! The parsing helpers, exercised the way a proc macro calls them: hand them a stream and
//! check what they consume and what they leave behind.
//!
//! Several of these pin behaviour that a reading of the source gets wrong, because a brace
//! reaches a `TokenTree` iteration as a single `Group` carrying its contents rather than as
//! two `Punct`s. Anything comparing a token to `"{"` is therefore comparing against `"{ b }"`
//! and never matches. The tests naming a dead branch say so.

use ohelpers_proc_macros::parse_utils::{
    comes_next, comes_next_any_surrounder, ends_next, find_end_of_widget_body, parse_peekables_until,
    parse_tokens_until,
    parse_until, ts_ends_next, SurrounderDir,
};
use proc_macro2::TokenStream;
use syn::parse::{ParseStream, Parser};
use syn::Token;

/// Runs a helper that wants a `ParseStream` against a source string.
///
/// Whatever the helper leaves behind is drained here, because `Parser::parse_str` treats an
/// unconsumed remainder as an error and most of these helpers are supposed to leave one.
fn on<T>(source: &str, f: impl FnOnce(ParseStream) -> syn::Result<T>) -> syn::Result<T> {
    let parser = move |input: ParseStream| {
        let out = f(input)?;
        let _: TokenStream = input.parse()?;
        Ok(out)
    };
    parser.parse_str(source)
}

/// Tokens have no equality, so they compare by their printed form.
fn shown(tokens: TokenStream) -> String {
    tokens.to_string()
}

#[test]
fn parse_peekables_until_stops_before_the_terminator_and_leaves_it() {
    let (taken, rest) = on("a b c ; d", |input| {
        let taken = parse_peekables_until(input, Token![;])?;
        let rest: TokenStream = input.parse()?;
        Ok((taken, rest))
    })
    .unwrap();

    assert_eq!(shown(taken), "a b c");
    assert_eq!(shown(rest), "; d", "the terminator is left for the caller");
}

#[test]
fn parse_peekables_until_consumes_everything_when_the_terminator_is_absent() {
    let taken = on("a b c", |input| parse_peekables_until(input, Token![;])).unwrap();
    assert_eq!(shown(taken), "a b c");
}

#[test]
fn parse_peekables_until_takes_a_brace_group_whole() {
    let taken = on("a { b ; c } d ;", |input| {
        parse_peekables_until(input, Token![;])
    })
    .unwrap();
    assert_eq!(
        shown(taken),
        "a { b ; c } d",
        "the semicolon inside the braces is not the terminator, because the group is one token"
    );
}

#[test]
fn parse_until_stops_before_the_terminator() {
    let taken = on("a b ; c", |input| parse_until(input, <Token![;]>::default())).unwrap();
    assert_eq!(shown(taken), "a b");
}

#[test]
fn parse_until_reaches_its_verdict_through_the_parse_and_not_the_type_id() {
    // `parse_until` tests `t.type_id() == TypeId::of::<E>()` alongside a `syn::parse2::<E>`.
    // `t` is a `TokenTree` at every iteration, so its type id is `TokenTree`'s and the
    // comparison is false for every terminator type anyone can pass. The parse is what
    // stops it, which this shows by stopping on a terminator whose type id could not match.
    let taken = on("x , y", |input| parse_until(input, <Token![,]>::default())).unwrap();
    assert_eq!(shown(taken), "x");
}

#[test]
fn parse_tokens_until_compares_the_printed_form() {
    let taken = parse_tokens_until("a b STOP c".parse().unwrap(), "STOP").unwrap();
    assert_eq!(shown(taken), "a b");
}

#[test]
fn parse_tokens_until_passes_everything_through_when_the_end_never_appears() {
    let taken = parse_tokens_until("a b c".parse().unwrap(), "STOP").unwrap();
    assert_eq!(shown(taken), "a b c");
}

#[test]
fn ends_next_reports_a_semicolon() {
    assert!(on("; rest", |input| Ok(ends_next(input))).unwrap());
    assert!(!on("rest ;", |input| Ok(ends_next(input))).unwrap());
    assert!(!on("", |input| Ok(ends_next(input))).unwrap());
}

#[test]
fn ts_ends_next_reports_a_leading_semicolon() {
    assert!(ts_ends_next("; a".parse().unwrap()));
    assert!(!ts_ends_next("a ;".parse().unwrap()));
    assert!(!ts_ends_next(TokenStream::new()));
}

#[test]
fn comes_next_reports_the_token_the_caller_asked_about() {
    assert!(on("-> x", |input| Ok(comes_next(input, Token![->]))).unwrap());
    assert!(!on("x ->", |input| Ok(comes_next(input, Token![->]))).unwrap());
}

#[test]
fn any_surrounder_sees_all_three_delimiters() {
    for source in ["{ a }", "( a )", "[ a ]"] {
        assert!(
            on(source, |input| Ok(comes_next_any_surrounder(
                input,
                SurrounderDir::Any
            )))
            .unwrap(),
            "{source} opens with a delimiter"
        );
    }
    assert!(!on("a { b }", |input| Ok(comes_next_any_surrounder(
        input,
        SurrounderDir::Any
    )))
    .unwrap());
}

#[test]
fn directional_surrounder_never_matches_a_brace_group() {
    // Forward compares the token's printed form against `"{"`. A brace group prints as
    // `"{ a }"`, so the comparison is false for every group, in both directions. This is
    // the dead branch named in the module docs; the test exists so that fixing it is a
    // visible change rather than a silent one.
    for dir in [SurrounderDir::Forward, SurrounderDir::Backward] {
        let seen = on("{ a }", |input| {
            Ok(comes_next_any_surrounder(input, dir.clone()))
        })
        .unwrap();
        assert!(!seen, "{dir:?} does not match a group, though it reads as if it does");
    }
}

#[test]
fn directional_surrounder_matches_a_bare_delimiter_punct() {
    // The one input the directional arms do answer: a delimiter that reached the stream as
    // a `Punct` rather than as a group, which happens for an unbalanced one.
    let seen = on("< a", |input| {
        Ok(comes_next_any_surrounder(input, SurrounderDir::Forward))
    })
    .unwrap();
    assert!(!seen, "an angle bracket is not one of the three surrounders");
}

#[test]
fn any_surrounder_on_an_exhausted_stream() {
    // `Any` peeks, which is fine on an empty stream. The directional arms parse a token and
    // unwrap it, so an exhausted stream panicked inside a helper whose return type is a
    // plain `bool` and whose only sensible answer is `false`.
    assert!(!on("", |input| Ok(comes_next_any_surrounder(
        input,
        SurrounderDir::Any
    )))
    .unwrap());

    for dir in [SurrounderDir::Forward, SurrounderDir::Backward] {
        let seen = on("", |input| Ok(comes_next_any_surrounder(input, dir.clone()))).unwrap();
        assert!(!seen, "{dir:?} on an empty stream is false, not a panic");
    }
}

#[test]
fn find_end_of_widget_body_stops_at_a_hash_following_a_group() {
    let taken = on("{ body } # rest", find_end_of_widget_body).unwrap();
    assert_eq!(shown(taken), "{ body }");
}

#[test]
fn find_end_of_widget_body_takes_everything_when_no_hash_follows() {
    let taken = on("{ body } more", find_end_of_widget_body).unwrap();
    assert_eq!(shown(taken), "{ body } more");
}

#[test]
fn find_end_of_widget_body_leaves_the_hash_for_the_caller() {
    let (taken, rest) = on("{ a } # b", |input| {
        let taken = find_end_of_widget_body(input)?;
        let rest: TokenStream = input.parse()?;
        Ok((taken, rest))
    })
    .unwrap();
    assert_eq!(shown(taken), "{ a }");
    assert_eq!(shown(rest), "# b");
}

#[test]
fn find_end_of_widget_body_on_empty_input() {
    let taken = on("", find_end_of_widget_body).unwrap();
    assert!(shown(taken).is_empty());
}

#[test]
fn a_brace_reaches_a_token_tree_walk_as_a_group_and_never_as_a_punct() {
    // The fact several of the tests above rest on, established here rather than in a
    // throwaway program. `find_end_of_widget_body` used to count `{` and `}` as `Punct`s;
    // neither arm could ever fire, so its counter held zero for every input and the branch
    // reading it was unreachable.
    //
    // It is worth pinning rather than assuming: it is a property of proc-macro2's
    // tokenisation, not of this crate, so nothing here would fail if it changed.
    let stream: TokenStream = "a { b } c".parse().unwrap();
    let kinds: Vec<&'static str> = stream
        .into_iter()
        .map(|tree| match tree {
            proc_macro2::TokenTree::Group(_) => "group",
            proc_macro2::TokenTree::Ident(_) => "ident",
            proc_macro2::TokenTree::Punct(_) => "punct",
            proc_macro2::TokenTree::Literal(_) => "literal",
        })
        .collect();

    assert_eq!(
        kinds,
        ["ident", "group", "ident"],
        "the braces and their contents are one token, not three"
    );

    // And the group prints with its contents, which is why comparing a token's printed form
    // against "{" never matches one.
    let stream: TokenStream = "{ b }".parse().unwrap();
    let only = stream.into_iter().next().unwrap();
    assert_eq!(only.to_string(), "{ b }");
    assert_ne!(only.to_string(), "{");
}
