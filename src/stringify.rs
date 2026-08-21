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
        <$name as $crate::__ohelpersToken>::display()
    };
    (parsable $name:expr) => {
        {
            use $crate::stringify::StringableParsable;

            // Every path absolute. This arm named `quote` without a leading `::` and
            // reached `Box` and `to_string` through the prelude, so it resolved in a crate
            // that depends on quote under that exact name and is not `#![no_std]`, and
            // nowhere else. The neighbouring arm was already written this way, which is
            // what made the difference easy to miss.
            let boxed: $crate::__ohelpers_box::Box<dyn $crate::__ohelpersToTokens> =
                $crate::__ohelpers_box::Box::new($name.clone());
            $crate::__ohelpers_string::ToString::to_string(
                &<StringableParsable as ::core::convert::From<
                    $crate::__ohelpers_box::Box<dyn $crate::__ohelpersToTokens>,
                >>::from(boxed),
            )
        }
    };
    (parsable ty $name:expr) => {
        {
            // `to_string` through this crate rather than through the prelude, for the same
            // reason as the arm above: the prelude is the consumer's, and a `#![no_std]`
            // one does not have it.
            let name = $crate::__ohelpers_string::ToString::to_string(
                &$crate::__ohelpersToTokens::into_token_stream($name.clone()),
            );
            // Trimmed, because a rendered path reads `a :: b :: C` and the raw segment
            // would carry the space that separated it from the `::`.
            $crate::__ohelpers_string::ToString::to_string(
                name.rsplit("::").next().unwrap_or(&name).trim(),
            )
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

/// Unwraps the box rather than wrapping it.
///
/// This was removed during a cleanup on the grounds that it needed an unsized `T` to be
/// reached, which is false: with `T = TokenStream` it is reachable by type annotation, and
/// removing it turned `StringableParsable::<TokenStream>::from(Box::new(tokens))` from a
/// conversion into a type error. Worse than the error, a caller using `.into()` without an
/// annotation silently got `StringableParsable<Box<T>>` instead.
///
/// It does overlap with the blanket impl above for a boxed argument, so an unannotated
/// `from` on a `Box` is ambiguous. That is a papercut a turbofish settles, and it is a
/// smaller cost than not having the conversion:
///
/// ```
/// use ohelpers_proc_macros::stringify::StringableParsable;
/// use quote::quote;
///
/// // The box is unwrapped: the wrapped type is `TokenStream`, not `Box<TokenStream>`.
/// let unwrapped: StringableParsable<proc_macro2::TokenStream> =
///     StringableParsable::from(Box::new(quote! { a + b }));
/// assert_eq!(unwrapped.to_string(), "a + b");
/// ```
impl<T: ToTokens> From<Box<T>> for StringableParsable<T> {
    fn from(value: Box<T>) -> Self {
        StringableParsable(*value)
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
