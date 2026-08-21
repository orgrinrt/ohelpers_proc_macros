use std::fmt::{Display, Formatter};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Peek;
use syn::token::Token;

#[macro_export]
macro_rules! token_name {
    (ty $name:ty) => {
        // `split` always yields at least one item, so the last one always exists.
        ::core::any::type_name::<$name>()
            .rsplit("::")
            .next()
            .unwrap_or(::core::any::type_name::<$name>())
    };
    (peekable ty $name:ty) => {
        <$name as ::syn::token::Token>::display()
    };
    (parsable $name:expr) => {
        {
            use $crate::stringify::StringableParsable;
            use quote::ToTokens;

            let name = (<StringableParsable as From<Box<dyn ToTokens>>>::from(Box::new($name.clone
            ()))
            ).to_string();
            name
        }
    };
    (parsable ty $name:expr) => {
        {
            use ::quote::ToTokens;

            let name = $name.clone().into_token_stream().to_string();
            // Trimmed, because a rendered path reads `a :: b :: C` and the raw segment
            // would carry the space that separated it from the `::`.
            name.rsplit("::").next().unwrap_or(&name).trim().to_string()
        }
    };
    (ident $name:ident) => {
        {
            $crate::token_name!(display $name)
        }
    };
    (display $name:expr) => {
        {
            let name = $name.to_string();
            name
        }
    };
    (peekable $name:expr) => {
        {
            use $crate::stringify::peekable_as_string;
            

            peekable_as_string($name)
        }
    };
}

pub struct StringableParsable<T: ToTokens = Box<dyn ToTokens>>(T);

pub fn parsable_as_string<T: ToTokens>(token: T) -> String {
    token.to_token_stream().to_string()
}

impl<T: ToTokens> ToTokens for StringableParsable<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // The wrapped value, not `(*self)`. Dereferencing a `&StringableParsable<T>` gives
        // a `StringableParsable<T>`, so the previous line called this method again and
        // overflowed the stack the first time anybody used the type as `ToTokens`.
        // `unconditional_recursion` had been reporting it.
        self.0.to_tokens(tokens)
    }
}

impl<T: ToTokens> From<T> for StringableParsable<T> {
    fn from(value: T) -> Self {
        StringableParsable(value)
    }
}

impl<T: ToTokens> Display for StringableParsable<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", parsable_as_string(self)))
    }
}

pub fn peekable_as_string<T: Peek>(_token: T) -> String {
    T::Token::display().to_string()
}
