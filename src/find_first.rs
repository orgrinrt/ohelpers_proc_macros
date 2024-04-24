use std::fmt::{Debug, Display, Formatter};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::__private::TokenStream2;
use syn::Error;
use syn::parse::{Parse, ParseStream, Peek};
use syn::token::Token;

use debug_helpers::debug_file;

use crate::{discard_next_token, token_name};

pub struct ParseFirst<P: Parse + Display>(P);

impl<P: Parse + Display> Display for ParseFirst<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl<P: Parse + Display + quote::ToTokens> ToTokens for ParseFirst<P> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = &self.0;
        quote!(#s).to_tokens(tokens);
    }
}

impl<P: Parse + Display + Token> Parse for ParseFirst<P> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug_file!(
            input,
            format!(
                "Starting to Parse first of type {} (input below)",
                token_name!
            (ty P)
            )
        );
        let original_input = input.fork();
        while !input.is_empty() {
            let result: Option<P> = input.parse::<P>().ok();
            if let Some(p) = result {
                debug_file!(
                    !"Finished parsing first of type {}, it was this: {}",
                    token_name!(ty P),
                    token_name!(ident p)
                );
                let _: TokenStream2 = input.parse()?;
                return Ok(Self(p));
            }
            discard_next_token!(input);
        }
        debug_file!(
            !"COULD NOT PARSE first of type {}, what was left was this: {}",
            token_name!(ty P),
            input
        );
        return Err(Error::new(
            input.span(),
            format!(
                "Stream `{}` contained no elem of type {}",
                original_input,
                P::display()
            )
            .as_str(),
        ));
    }
}

pub struct PeekFirst<P: Parse + Default>(P);

impl<P: Parse + Default + Token> Display for PeekFirst<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", P::display()))
    }
}

impl<P: Parse + Default + Token> Parse for PeekFirst<P> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        while !fork.is_empty() {
            let result: Option<P> = fork.parse::<P>().ok();
            if let Some(p) = result {
                return Ok(Self(p));
            }
            discard_next_token!(input);
        }
        return Err(Error::new(
            input.span(),
            format!(
                "Stream `{}` contained no elem of type {}",
                input,
                P::display()
            )
            .as_str(),
        ));
    }
}
