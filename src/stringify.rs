use std::fmt::{Display, Formatter};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Peek;
use syn::token::Token;

#[macro_export]
macro_rules! token_name {
    (ty $name:ty) => {
        {
            if std::any::type_name::<$name>().split("::").last().is_some() {
                std::any::type_name::<$name>().split("::").last().unwrap()
            } else {
                std::any::type_name::<$name>()
            }
        }
    };
    (peekable ty $name:ty) => {
        <$name>::display();
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
            use quote::ToTokens;

            let name = $name.clone().into_token_stream().to_string();
            let split =  name.split("::");
            let count = split.clone().count();
            if count > 0 {
                let last = split.clone().last();
                if last.is_some() {
                    split.last().unwrap().to_string()
                }
                else {
                    name
                }
            }
            else {
                name
            }
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
            use syn::parse::Peek;

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
        (*self).to_tokens(tokens)
    }
}

impl<T: ToTokens> From<T> for StringableParsable<T> {
    fn from(value: T) -> Self {
        StringableParsable(value)
    }
}

impl<T: ToTokens> From<Box<T>> for StringableParsable<T> {
    fn from(value: Box<T>) -> Self {
        StringableParsable(*value)
    }
}

impl<T: ToTokens> Display for StringableParsable<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", parsable_as_string(&*self)))
    }
}

pub fn peekable_as_string<T: Peek>(token: T) -> String {
    T::Token::display().to_string()
}
