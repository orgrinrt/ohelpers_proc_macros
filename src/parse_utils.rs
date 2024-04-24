use std::any::{Any, TypeId};
use std::fmt::Display;

use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseBuffer, ParseStream, Parser, Peek};
use syn::token::Token;
use syn::Token;

use crate::{__pmmh_debug_file, token_name};

pub struct AnyParsable(TokenTree);

impl Parse for AnyParsable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let result = TokenTree::parse(input)?;
        Ok(Self(result))
    }
}

impl ToTokens for AnyParsable {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let tt = &self.0;
        let stream = tt.into_token_stream();
        tokens.extend(stream);
    }
}

pub fn find_end_of_widget_body(input: ParseStream) -> syn::Result<TokenStream2> {
    let mut output = TokenStream2::new();
    let mut braces_counter = 0;
    let lookahead_fork = input.fork();

    while !lookahead_fork.is_empty() {
        let lookahead = lookahead_fork.parse::<TokenTree>()?;
        let l = &lookahead;

        match l {
            TokenTree::Punct(punct) if punct.as_char() == '{' => {
                braces_counter += 1;
            },
            TokenTree::Punct(punct) if punct.as_char() == '}' => {
                braces_counter -= 1;

                if braces_counter <= 1 {
                    // Lookahead to check the next token
                    let next_fork = input.fork();
                    // Check if there's more tokens AND the next token is our delimiter
                    if !next_fork.is_empty() {
                        let next_token = next_fork.parse::<Token![#]>();
                        if let Ok(t) = next_token {
                            break;
                        }
                    }
                }
            },
            TokenTree::Punct(punct) if punct.as_char() == '#' => {
                if output.to_string().trim_end().ends_with("}") {
                    break;
                }
            },
            _ => {},
        }

        output.extend(Some(lookahead));
        // Consume the token from the input
        let _ = input.parse::<TokenTree>()?;
    }

    Ok(output)
}

pub fn parse_peekables_until<E: Peek>(input: ParseStream, end: E) -> syn::Result<TokenStream2> {
    let mut tokens = TokenStream2::new();
    while !input.is_empty() && !input.peek(end) {
        let next: TokenTree = input.parse()?;
        tokens.extend(Some(next));
    }
    Ok(tokens)
}

pub fn ends_next(input: ParseStream) -> bool {
    let end = Token![;];
    if input.peek(end) {
        return true;
    }
    false
}

pub fn ts_ends_next(input: TokenStream2) -> bool {
    input.to_string().trim_start().starts_with(";")
}

pub fn comes_next<E: Peek>(input: ParseStream, starter: E) -> bool {
    if input.peek(starter) {
        return true;
    }
    false
}

#[derive(PartialEq, Debug, Clone)]
pub enum SurrounderDir {
    Forward,
    Backward,
    Any,
}

pub fn comes_next_any_surrounder(input: &ParseBuffer, dir: SurrounderDir) -> bool {
    if dir == SurrounderDir::Any {
        if input.peek(syn::token::Paren)
            || input.peek(syn::token::Brace)
            || input.peek(syn::token::Bracket)
        {
            return true;
        }
        return false;
    } else {
        let fork = input.fork();
        let tt = fork.parse::<TokenTree>().unwrap();
        __pmmh_debug_file!(!"\t>> found token: {}", tt.to_string());
        if dir == SurrounderDir::Forward {
            if tt.to_string().trim() == "{"
                || tt.to_string().trim() == "("
                || tt.to_string().trim() == "["
            {
                return true;
            }
            return false;
        } else if dir == SurrounderDir::Backward {
            if tt.to_string().trim() == "}"
                || tt.to_string().trim() == ")"
                || tt.to_string().trim() == "]"
            {
                return true;
            }
            return false;
        }
        false
    }
}

pub fn parse_until<E: Token + Parse + Default + 'static>(
    input: ParseStream,
    end: E,
) -> syn::Result<TokenStream2> {
    let mut tokens = TokenStream2::new();
    while !input.is_empty() {
        let fork = input.fork();
        let tt: Option<TokenTree> = fork.parse().ok();
        __pmmh_debug_file!(
            !"\t\t << Found TokenTree `{}`",
            tt.to_token_stream().to_string()
        );
        if let Some(t) = tt {
            let ts = t.to_token_stream();
            let e: Option<E> = syn::parse2::<E>(ts).ok();
            if t.type_id() == TypeId::of::<E>() || e.is_some() {
                break;
            }
        }
        let next: TokenTree = input.parse()?;
        tokens.extend(Some(next));
    }
    Ok(tokens)
}

pub fn parse_peeks_until<E: Token + Clone + ToTokens + 'static>(
    input: ParseStream,
    end: E,
) -> syn::Result<TokenStream2> {
    let mut tokens = TokenStream2::new();
    while !input.is_empty() {
        let fork = input.fork();
        let tt: Option<TokenTree> = fork.parse().ok();
        if let Some(t) = tt {
            __pmmh_debug_file!(
                !"Comparing {} vs {}",
                t.to_string(),
                token_name!(peekable ty E)
            );
            if t.to_string() == token_name!(parsable end) {
                break;
            }
        }
        let next: Option<TokenTree> = input.parse().ok();
        tokens.extend(next);
    }
    Ok(tokens)
}

pub fn parse_tokens_until<E: Display>(input: TokenStream2, end: E) -> syn::Result<TokenStream2> {
    let mut tokens = TokenStream2::new();
    for token in input.into_iter() {
        if token.to_string().trim() == end.to_string().trim() {
            break;
        }
        tokens.extend(Some(token));
    }
    Ok(tokens)
}

#[macro_export]
macro_rules! discard_next_token {
    ($input:expr) => {{
        use syn::parse::Parse;
        let discard = $input.parse::<proc_macro2::TokenTree>().ok();
        if discard.is_some() {
            use $crate::__pmmh_debug_file;
            __pmmh_debug_file!(!"\t\t<<< Discarding {}", discard.unwrap().to_string());
        } else {
            panic!("Attempted to discard from stream, but found no tokens");
        }
    }};
    ($input:expr, _) => {{
        use $crate::parse_utils::AnyParsable;
        let _: AnyParsable = $input.parse()?;
    }};
}

#[macro_export]
macro_rules! compare_tokens {
    (ty $a:ty, $b:ty) => {
        $crate::token_name!(ty $a) == $crate::token_name!(ty $b)
    };
    (parsable $a:expr, $b:expr) => {
        $crate::token_name!(parsable $a) == $crate::token_name!(parsable $b)
    };
    (peekable $a:expr, $b:expr) => {
        $crate::token_name!(peekable $a) == $crate::token_name!(peekable $b)
    };
}

#[macro_export]
macro_rules! unwrap {
    (braces $content:ident in $cursor:expr) => {
        {
            let _res = syn::__private::parse_braces(&$cursor);
            match _res {
                syn::__private::Ok(parens) => {
                    $content = parens.content;
                }
                syn::__private::Err(e) => {
                    use $crate::token_name;
                    panic!("Failed to unwrap braces for `{}`, with error: {}",
                    token_name!(display $cursor), e);
                }
            }
        }
    };
    (brackets $content:ident in $cursor:expr) => {
        {
            let _res = syn::__private::parse_brackets(&$cursor);
            match _res {
                syn::__private::Ok(parens) => {
                    $content = parens.content;
                }
                syn::__private::Err(e) => {
                    use $crate::token_name;
                    panic!("Failed to unwrap brackets for `{}`, with error: {}",
                    token_name!(display $cursor), e);
                }
            }
        }
    };
    (parens $content:ident in $cursor:expr) => {
        {
            let _res = syn::__private::parse_parens(&$cursor);
            match _res {
                syn::__private::Ok(parens) => {
                    $content = parens.content;
                }
                syn::__private::Err(e) => {
                    use $crate::token_name;
                    panic!("Failed to unwrap parens for `{}`, with error: {}",
                    token_name!(display $cursor), e);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! unwrap_body {
    (explicit peekable $input:expr, $var:ident, $delim:expr) => {
        {
            use $crate::stringify::StringableParsable;
            use $crate::token_name;
            use $crate::compare_tokens;
            use $crate::unwrap;

            if compare_tokens!(peekable $delim, syn::token::Brace) {
                unwrap!(braces $var in $input);
            } else if compare_tokens!(peekable $delim, syn::token::Paren)  {
                unwrap!(parens $var in $input);
            } else if compare_tokens!(peekable $delim, syn::token::Bracket)  {
                unwrap!(brackets $var in $input);
            } else {
                panic!("The delim is not appropriate for a body. Use either paren, \
                brace or bracket, instead of {}", token_name!(peekable $delim))
            }
        }
    };
    (peekable $input:expr, $var:ident, $delim:expr) => {
        let mut $var;
        $crate::unwrap_body!(explicit peekable $input, $var, $delim);
    };
    (explicit parsable $input:expr, $var:ident, $delim:expr) => {
        {
            use $crate::stringify::StringableParsable;
            use $crate::token_name;
            use $crate::compare_tokens;
            use $crate::unwrap;

            if compare_tokens!(parsable $delim, syn::token::Brace) {
                unwrap!(braces $var in $input);
                if let Err(e) = $var {
                    panic!("Failed to unwrap braced for {}, with error: {}", token_name!(peekable
                    $delim), e)
                }
            } else if compare_tokens!(parsable $delim, syn::token::Paren)  {
                unwrap!(parens $var in $input);
                if let Err(e) = $var {
                    panic!("Failed to unwrap parens for {}, with error: {}", token_name!(peekable
                    $delim), e)
                }
            } else if compare_tokens!(parsable $delim, syn::token::Bracket)  {
                unwrap!(brackets $var in $input);
                if let Err(e) = $var {
                    panic!("Failed to unwrap brackets for {}, with error: {}", token_name!(peekable
                    $delim), e)
                }
            } else {
                panic!("The delim is not appropriate for a body. Use either paren, \
                brace or bracket, instead of {}", token_name!(peekable $delim))
            }
        }
    };
    (parsable $input:expr, $var:ident, $delim:expr) => {
        let mut $var;
        $crate::unwrap_body!(explicit parsable $input, $var, $delim);
    };
    (explicit ty $input:expr, $var:ident, $delim:ty) => {
        {
            use $crate::stringify::StringableParsable;
            use $crate::token_name;
            use $crate::compare_tokens;

            if compare_tokens!(ty $delim, syn::token::Brace) {
                syn::braced!($var in $input);
            } else if compare_tokens!(ty $delim, syn::token::Paren)  {
                syn::parenthesized!($var in $input);
            } else if compare_tokens!(ty $delim, syn::token::Bracket)  {
                syn::bracketed!($var in $input);
            } else {
            panic!("The delim is not appropriate for a body. Use either paren, \
            brace or bracket, instead of {}", token_name!(ty $delim))
            }
        }
    };
    (ty $input:expr, $var:ident, $delim:ty) => {
        let mut $var;
        $crate::unwrap_body!(explicit ty $input, $var, $delim);
    };
}

#[macro_export]
macro_rules! unwrap_input {
    ($input:expr, $new:ident) => {
        $crate::unwrap_input!($input, $new, syn::token::Paren);
    };
    ($input:expr, $new:ident, $token:ty) => {
        paste! {
            let mut $new: syn::parse::ParseStream;
            let mut [<_ $new>];
            {
                use $crate::parse_utils::comes_next;
                $crate::__pmmh_debug_file!(!"\n>>Unwrapping this input by delim `{}`: \n`{}`",
                $crate::token_name!(peekable $token), $input);
                if comes_next($input, $token) {
                    $crate::unwrap_body!(explicit ty $input, [<_ $new>], $token);
                    $crate::__pmmh_debug_file!(!"\n>>Finished unwrapping, got this as result: \n`{}`",
                &[<_ $new>]);
                    $new = &[<_ $new>];
                } else {
                    $crate::__pmmh_debug_file!(!"\n>>Finished unwrapping, there was nothing to unwrap \
:-). The remaining input is below: \n`{}`", $input);
                    $new = $input;
                }
            }
        }
    };
}

#[macro_export]
macro_rules! try_get_tuple_params {
    ($input:expr, $var:ident) => {
        $crate::try_get_tuple_params!($input, $var, Params);
    };
    ($input:expr, $var:ident, $punct_set:ty) => {
        let mut $var: Option<$punct_set> = None;
        {
            use syn::parse::Parse;
            use syn::parse::Peek;

            if $input.peek(token::Paren) || $input.peek2(token::Paren) {
                if $input.peek2(token::Paren) {
                    $crate::discard_next_token!($input);
                }
                let tuple_args_stream;
                syn::parenthesized!(tuple_args_stream in $input);
                $var = tuple_args_stream.parse().ok();
            }
        }
    };
}

#[macro_export]
macro_rules! try_get_trails {
    ($input:expr, $var:ident) => {
        $crate::try_get_trails!($input, $var, $crate::declaration::Trails);
    };
    ($input:expr, $var:ident, $punct_set:ty) => {
        let mut $var: Option<$punct_set> = None;
        'wrap: {
            use syn::parse::Parse;
            use syn::parse::Peek;

            let mut is_valid = false;
            if $input.peek(syn::Token![:]){
                is_valid = true;
                $crate::discard_next_token!($input);
            }
            if !is_valid {
                $crate::__pmmh_debug_file!(!"{} did not have any trails! skipping!", $input);
                break 'wrap;
            }
            $crate::__pmmh_debug_file!(!"Trails were valid! What remains is: {}", $input);
            let ts = parse_peekables_until($input, syn::token::Brace)?;
            $var = syn::parse2::<$punct_set>(ts).ok();
        }
    };
}

// #[macro_export]
// macro_rules! ts_to_buffer {
//     ($tokens:expr, $var:ident) => {
//         let $var = {
//             use syn::parse::ParseBuffer;
//             use syn::buffer::TokenBuffer;
//             use syn::punctuated::Punctuated;
//             use std::rc::Rc;
//             use std::cell::Cell;
//             use std::mem;
//             use syn::buffer::Cursor;
//             use std::marker::PhantomData;
//
//             let tokens = TokenBuffer::new2($tokens.into());
//             let scope = Span::call_site();
//             let cursor = tokens.begin();
//             // let unexpected = Rc::new(Cell::new(0));
//             syn::new_parse_buffer(
//                 scope,
//                 cursor,
//                 syn::parse::Unexpected::None)
//         };
//     };
// }

#[macro_export]
macro_rules! try_get_body {
    (ts $input:ident, $var:ident) => {
        $crate::try_get_body!(ts $input, $var, syn::token::Brace)
    };
    (ts $input:ident, $var:ident, $delim:expr) => {
         let parser = |input: ParseStream| -> syn::Result<TokenStream2> {
            try_get_body!(input, body, $delim);
            body.parse::<TokenStream2>()
        };
        let $var = parse_macro_input!($input with parser);
    };
    ($input:expr, $var:ident) => {
        $crate::try_get_body!($input, $var, syn::token::Brace)
    };
    ($input:expr, $var:ident, $delim:expr) => {
        let $var: syn::parse::ParseBuffer;
        $crate::try_get_body!(explicit $input, $var, $delim);
    };
    (explicit $input:expr, $var:ident, $delim:expr) => {
        {
            use syn::parse::Parse;
            use syn::parse::Peek;

            let mut is_valid = false;
            if $input.peek($delim)  {
                is_valid = true;
            }
            if !is_valid {
                panic!("Expected body to begin with a `{}` token \
                on input: \n`{}`\n", $crate::token_name!(peekable $delim), $input);
            }
            $crate::unwrap_body!(explicit peekable $input, $var, $delim);
        }
    };
}

#[macro_export]
macro_rules! surround {
    ($keyword:ident, $delim:ty, $from:expr, $to:ident) => {
        <$delim>::default().surround(&mut $to, |mut inner| {
            inner.extend(keyword::$keyword::default().into_token_stream());
            $from.clone().to_tokens(&mut inner);
        });
    };
    ($delim:ty, $from:expr, $to:ident) => {
        <$delim>::default().surround(&mut $to, |mut inner| {
            $from.clone().to_tokens(&mut inner);
        });
    };
}
