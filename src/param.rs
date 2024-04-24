use std::fmt::{Display, Formatter};

use debug_helpers::debug_file;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::__private::TokenStream2;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{parenthesized, token, Expr, Ident as IdentSyn, Token, Type};

use crate::define::Define;
use crate::find_first::ParseFirst;
use crate::parse_utils::AnyParsable;
use crate::punct_set::PunctSet;
use crate::{surround, EMPTY, EMPTY_STR};

pub type Params = PunctSet<Param>;

impl Params {
    pub fn as_regular_let_decls(&self) -> TokenStream2 {
        let mut result = TokenStream2::new();
        let vals = &self.vals;
        for val in vals {
            let let_decl = val.as_regular_let_decl();
            let_decl.to_tokens(&mut result);
        }
        result
    }

    pub fn as_regular_let_decls_with_value_from_samed_name_var(&self) -> TokenStream2 {
        let mut result = TokenStream2::new();
        let vals = &self.vals;
        let vals_len = vals.len();
        for (idx, val) in vals.iter().enumerate() {
            let let_decl = val.as_regular_let_decl_without_end_punct();
            let val: ParseFirst<Ident> =
                syn::parse2(let_decl.clone()).expect("Expected a valid ident for the let_decl");
            let end = if idx >= vals_len - 1 { quote!() } else { quote!(,) };
            let q = quote! {
                #let_decl = #val.clone() #end
            };
            q.to_tokens(&mut result);
        }
        result
    }

    pub fn as_regular_let_decls_with_value_from_samed_name_self_var(&self) -> TokenStream2 {
        let mut result = TokenStream2::new();
        let vals = &self.vals;
        let vals_len = vals.len();
        for (idx, val) in vals.iter().enumerate() {
            let let_decl = val.as_regular_let_decl_without_end_punct();
            let val: ParseFirst<Ident> =
                syn::parse2(let_decl.clone()).expect("Expected a valid ident for the let_decl");
            let end = if idx >= vals_len - 1 { quote!() } else { quote!(,) };
            let q = quote! {
                #let_decl = self.#val.clone() #end
            };
            q.to_tokens(&mut result);
        }
        result
    }

    pub fn as_value_decls_with_value_from_samed_name_var(&self) -> TokenStream2 {
        let mut result = TokenStream2::new();
        let vals = &self.vals;
        let vals_len = vals.len();
        for (idx, val) in vals.iter().enumerate() {
            let let_decl = val.as_define();
            let val = &let_decl.ident;
            let end = if idx >= vals_len - 1 { quote!() } else { quote!(,) };
            let q = quote! {
                #val: #val.clone() #end
            };
            q.to_tokens(&mut result);
        }
        result
    }
}

/// e.g:
/// ```
/// pub mut foo: Bar(fish: Fash = default()) = Bar::new(xyz)
/// mut a: u8 = 1
/// pub b: String = "aye".to_string()
/// X(y) = z
/// mut X(y)
/// Z
/// ```
#[derive(Clone)]
pub struct Param {
    pub publicity:   Option<Token![pub]>,
    pub mutability:  Option<Token![mut]>,
    pub name:        Option<Ident>,
    pub ty:          Type,
    pub tuple_args:  Option<PunctSet<TokenStream2, Token![,]>>,
    pub default_val: Option<(Token![=], Expr)>,
}

impl Param {
    pub fn as_regular_let_decl(&self) -> TokenStream2 {
        let publicity = &self.publicity; // NOTE: prob doesn't make sense here?
        let mutability = &self.mutability;
        let name = &self.name;
        let ty = &self.ty;
        // let tuple_args = &self.tuple_args; // NOTE: this doesn't make sense here I
        // think?
        let default_val = &self.default_val;
        let ts = match default_val {
            Some((_, default_val)) => {
                quote! {
                    let #mutability #name: #ty = #default_val ;
                }
            },
            None => {
                quote! {
                    let #mutability #name: #ty ;
                }
            },
        };
        ts
    }

    pub fn as_regular_let_decl_without_end_punct(&self) -> TokenStream2 {
        let publicity = &self.publicity; // NOTE: prob doesn't make sense here?
        let mutability = &self.mutability;
        let name = &self.name;
        let ty = &self.ty;
        // let tuple_args = &self.tuple_args; // NOTE: this doesn't make sense here I
        // think?
        let default_val = &self.default_val;
        let ts = match default_val {
            Some((_, default_val)) => {
                quote! {
                    let #mutability #name: #ty = #default_val
                }
            },
            None => {
                quote! {
                    let #mutability #name: #ty
                }
            },
        };
        ts
    }

    pub fn as_define(&self) -> Define {
        let name = self.name.clone();
        let value = self.ty.clone();
        Define {
            ident: name.expect(
                "Expected Param that can be converted to a Define, to have a \
            name identity (i.e `name: Type`, wherein the `name` is missing here",
            ),
            value: value.into_token_stream(),
        }
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}{}\
        {}{}{}",
            if self.publicity.is_some() { "pub " } else { EMPTY },
            if self.mutability.is_some() { "mut " } else { EMPTY },
            if self.name.is_some() {
                format!("{}: ", self.name.clone().unwrap())
            } else {
                EMPTY_STR.clone()
            },
            self.ty.to_token_stream().to_string(),
            if self.tuple_args.is_some() {
                format!("({})", self.tuple_args.clone().unwrap())
            } else {
                EMPTY_STR.clone()
            },
            if self.default_val.is_some() {
                format!(
                    " = {}",
                    self.default_val
                        .clone()
                        .unwrap()
                        .1
                        .to_token_stream()
                        .to_string()
                )
            } else {
                EMPTY_STR.clone()
            },
        ))
    }
}

impl ToTokens for Param {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut result = TokenStream2::new();
        let mut s = self;
        if let Some(v) = s.publicity {
            v.to_tokens(&mut result);
        }
        if let Some(v) = s.mutability {
            v.to_tokens(&mut result);
        }
        if let Some(v) = &s.name {
            v.clone().to_tokens(&mut result);
            token::Colon::default().to_tokens(&mut result);
        }
        self.ty.to_tokens(&mut result);
        if let Some(v) = &s.tuple_args {
            surround!(token::Paren, v, result);
        }
        if let Some(v) = &s.default_val {
            v.0.to_tokens(&mut result);
            v.clone().1.to_tokens(&mut result);
        }
        tokens.extend(result);
    }
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        debug_file!(input, "Starting to parse a Param, initial input below:");

        let publicity: Option<Token![pub]> = input.parse().ok();
        let mutability: Option<Token![mut]> = input.parse().ok();
        let mut name: Option<Ident> = None;
        let mut type_prefix: Option<Token![:]> = None;
        if input.peek2(Token![:]) && input.peek3(IdentSyn) {
            name = input.parse().ok();
            type_prefix = input.parse().ok();
        }
        if type_prefix.is_none() && name.is_some() {
            return Err(input.error(
                "Expected a : to denote the variable type (already had an \
            ident for var name)",
            ));
        }

        let _ty: Option<Type> = input.parse().ok();
        if _ty.is_none() {
            let some: TokenTree = input.parse()?;
            let msg = format!("Expected a Type for a Param, but got: `{}`", some);
            debug_file!(!"\tERROR: {}", msg);
            return Err(input.error(msg));
        }
        let ty = _ty.expect("Expected a Param to have a Type");
        debug_file!(
            !"\t<<< Found Type `{}` for the param",
            ty.clone().into_token_stream().to_string()
        );

        let mut tuple_args: Option<PunctSet<TokenStream2, Token![,]>> = None;
        if input.peek(token::Paren) || input.peek2(token::Paren) {
            if input.peek2(token::Paren) {
                let _: AnyParsable = input.parse()?;
            }
            let tuple_args_stream;
            parenthesized!(tuple_args_stream in input);
            tuple_args = tuple_args_stream.parse().ok();
        }

        let default_val: Option<(Token![=], Expr)> = if input.peek(Token![=]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        let result = Self {
            publicity,
            mutability,
            name,
            ty,
            tuple_args,
            default_val,
        };

        debug_file!(result, "Succesfully parsed a Param, value below:");
        debug_file!("\n");

        Ok(result)
    }
}
