use std::fmt::{Display, Formatter};

use debug_helpers::debug_file;
use derive_display::derive_display;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{token, Token};

use crate::punct_set::PunctSet;
use crate::TokenStream2;

pub type Defines = PunctSet<Define, Token![,]>;

impl Defines {
    pub fn empty() -> Self {
        Defines {
            vals:  Vec::new(),
            delim: Some(token::Comma::default()),
        }
    }
}

/// This is a simple definition field format, which consists of two components:
/// * `field_name` : `field_type`
///
/// E.g:
/// ```
/// foo: Bar
/// ```
/// Alternatively, it can be used to represent simple value assignments in
/// custom dsl, for example:
/// * `var_name` : `var_value`
///
/// E.g:
/// ```
/// thing: SomeCtor(cool)
/// ```
#[derive(Clone)]
pub struct Define {
    pub ident: Ident,
    pub value: TokenStream2,
}

impl Define {
    pub fn name(&self) -> String {
        self.ident.to_string().to_lowercase()
    }

    pub fn as_pub(&self) -> TokenStream2 {
        let s = self;
        quote!(pub #s)
    }
}

#[derive_display]
impl ToTokens for Define {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let value = &self.value;
        let q = quote! {
            #ident: #value
        };
        q.to_tokens(tokens);
    }
}

impl Parse for Define {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse().ok().expect("Expected an ident");
        let _ = input
            .parse::<TokenTree>()
            .expect("Expected a `:` to separate ident and value");
        let value: TokenStream2 = input.parse().ok().expect(
            "Expected a value after the \
        ident",
        );
        let result = Self {
            ident,
            value,
        };
        debug_file!(result, "Finishing parsin Define, value below:");
        Ok(result)
    }
}
