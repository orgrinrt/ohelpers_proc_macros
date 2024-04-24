use std::fmt::{Display, Formatter};

use debug_helpers::debug_file;
use paste::paste;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream};
use syn::{token, Token};

use crate::param::Params;
use crate::parse_utils::parse_peekables_until;
use crate::punct_set::PunctSet;
use crate::{
    discard_next_token,
    surround,
    try_get_trails,
    try_get_tuple_params,
    unwrap_input,
    EMPTY_STR,
};

pub type Trails<T: Parse = Declaration> = PunctSet<T, Token![+]>;

/// e.g:
/// ```
/// SomeType(a: u8 = 1, b: f32): TraitA + TraitB + TraitC
/// SomeType(X(y)): Foo + Bar
/// Foo(bar)
/// ```
/// NOTE: Also allows for a braced body that is just returned as a single
/// TokenStream2 ```
/// Foo(bar) {
///    // Something here
/// }
/// ```
#[derive(Clone)]
pub struct Declaration {
    pub name:   Option<Ident>,
    pub ty:     Ident,
    pub params: Option<Params>,
    pub trails: Option<Trails>,
    pub body:   Option<TokenStream2>,
}

impl Declaration {
    pub fn as_widget_build_block(&self) -> TokenStream2 {
        let ty = &self.ty;
        let params = &self.params;
        quote! {
            #ty::new(#params)
        }
    }
}

impl Display for Declaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}{}{}",
            if self.name.is_some() {
                format!("{}: ", self.name.clone().unwrap())
            } else {
                EMPTY_STR.clone()
            },
            self.ty.to_token_stream().to_string(),
            if self.params.is_some() {
                format!("({})", self.params.clone().unwrap())
            } else {
                EMPTY_STR.clone()
            },
            if self.trails.is_some() {
                format!(": {}", self.trails.clone().unwrap())
            } else {
                EMPTY_STR.clone()
            },
        ))
    }
}

impl ToTokens for Declaration {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut result = TokenStream2::new();
        let mut s = self;
        if let Some(v) = &s.name {
            self.ty.to_tokens(&mut result);
            token::Colon::default().to_tokens(&mut result);
        }
        self.ty.to_tokens(&mut result);
        if let Some(v) = &s.params {
            surround!(token::Paren, v, result);
        }
        if let Some(v) = &s.trails {
            result.extend(token::Colon::default().to_token_stream());
            v.clone().to_tokens(&mut result);
        }
        if let Some(v) = &s.body {
            surround!(token::Brace, v, result);
        }
        result.to_tokens(tokens);
    }
}

impl Parse for Declaration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        unwrap_input!(input, unwrapped_input);
        let mut name: Option<Ident> = None;
        if unwrapped_input.peek2(Token![:]) {
            name = unwrapped_input.parse().ok();
            discard_next_token!(unwrapped_input);
        }
        let ty: Ident = unwrapped_input
            .parse()
            .expect("Expected a Type for Declaration");
        debug_file!(!"Found ident, starting to parse Declaration for `{}`", ty);
        try_get_tuple_params!(unwrapped_input, params, Params);
        try_get_trails!(unwrapped_input, trails);
        if trails.is_some() {
            debug_file!(
                trails.clone().unwrap(),
                "Found trails for Declaration! (below)"
            );
        } else {
            debug_file!("Found no trails for Declaration :-(");
        }
        parse_peekables_until(unwrapped_input, token::Brace)?;
        let mut body = None;
        if !unwrapped_input.is_empty() {
            let fork = unwrapped_input.fork();
            body = fork.parse().ok();
        }
        Ok(Self {
            name,
            ty,
            params,
            trails,
            body,
        })
    }
}
