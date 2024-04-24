use std::fmt::{Display, Formatter, Pointer, Write};

use debug_helpers::debug_file;
use derive_display::derive_display;
use paste::paste;
use proc_macro2::TokenTree;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, Parser, Peek};
use syn::token::Token;
use syn::Token;

use crate::parse_utils::parse_until;
use crate::{token_name, unwrap_input, TokenStream2};

/// Any punctuated set of instances of type T, delimited by a token of type D
/// e.g
/// ```
/// here, is, stuff, de, limited, by_commas, each_being_an_ident
/// This + Is + Types + Delimited + By + Pluses
/// a::b::c::d::e::f::g
/// x?y?z
/// ```
#[derive(Clone)]
pub struct PunctSet<
    T: Parse + Display + ToTokens = TokenStream2,
    D: Token + Parse + Default = Token![,],
> {
    pub vals:  Vec<T>,
    pub delim: Option<D>,
}

#[derive_display]
impl<T: Parse + Display + ToTokens + Clone, D: Token + Parse + Default + ToTokens> ToTokens
    for PunctSet<T, D>
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let last = &self.vals.iter().last().expect(
            "Expected the PunctSet to contain at least one \
         element",
        );
        let mut vals: Vec<T> = Vec::new();
        let len = self.vals.iter().count();
        for (idx, val) in self.vals.iter().enumerate() {
            if idx >= len - 1 {
                break;
            }
            vals.push(val.clone());
        }
        let d = &self.delim;
        if vals.len() > 0 {
            let q = quote!(#(#vals #d)*);
            q.to_tokens(tokens);
        }
        last.to_tokens(tokens);
    }
}

impl<T: Parse + Display + ToTokens + 'static, D: Token + Parse + Default + 'static>
    syn::parse::Parse for PunctSet<T, D>
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        debug_file!(
            input,
            format!(
                "parsing a {:?} from a punctuated stream, by delim `{}`: input \
        below",
                token_name!(ty T),
                D::display()
            )
        );
        // let mut streams: Punctuated<TokenStream2, D> =
        // Punctuated::parse_separated_nonempty(input)?;
        unwrap_input!(input, unwrapped_input);
        let mut vec: Vec<T> = Vec::new();
        let mut delim: Option<D> = None;
        while !unwrapped_input.is_empty() {
            let (token, next) = unwrapped_input.cursor().token_tree().unwrap();
            let punct: Option<D> = syn::parse2(token.to_token_stream()).ok();
            if next.eof() && punct.is_some() {
                break;
            }
            debug_file!(!"Processing unwrapped_input `{}`", unwrapped_input);
            let stream: TokenStream2 = parse_until(unwrapped_input, D::default())?;
            debug_file!(
                !"Parsed until {}, what remains is: `{}`",
                D::display(),
                stream
            );
            let val: Option<T> = syn::parse2::<T>(stream.clone()).ok();
            if val.is_none() {
                let actual = syn::parse2::<TokenTree>(stream.clone()).ok().expect(format!("Found no `{}`, but expected there to be at least some TokenTree left in: `{}`",
                                                                                          token_name!(ty T),
                                                                                          stream
                                                                                              .to_string()).as_str());
                panic!(
                    "Expected a {}, but found {}",
                    token_name!(ty T),
                    token_name!(parsable actual)
                );
            }
            let v = val.unwrap();
            debug_file!(!"Found stream `{}`", v);
            if delim.is_none() {
                delim = unwrapped_input.parse().ok();
                debug_file!(!"Found a punct `{:?}`", D::display());
            }
            vec.push(v);
        }
        debug_file!(quote!(#(#vec),*), "Result of parsing PunctSet below:");
        Ok(Self {
            vals: vec,
            delim,
        })
    }
}
