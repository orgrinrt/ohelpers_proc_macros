// `alloc` rather than `std`, and re-exported rather than named at the expansion site.
//
// `Box` and `ToString` live in `alloc` and are re-exported by `std`, so these are the same
// two types either way. What the re-export buys is that a consumer never has to name either
// crate: the exported macros go through `$crate::__ohelpers_box`, which resolves wherever
// this crate does, so a `#![no_std]` consumer needs no `extern crate alloc;` of its own and
// a consumer that renamed its dependencies is unaffected.
//
// Naming them through the prelude is what the `token_name!(parsable ..)` arm used to do,
// and it resolved only in a consumer that was neither.
extern crate alloc;

#[doc(hidden)]
pub use ::alloc::boxed as __ohelpers_box;
#[doc(hidden)]
pub use ::alloc::string as __ohelpers_string;

// Same reasoning, one level out. `::quote::ToTokens` is absolute within the *consumer's*
// crate graph, so it resolves only where the consumer has a dependency spelled `quote`.
// A crate that renamed it got `could not find 'quote' in the list of imported crates`,
// spanned inside the macro. Reached through this crate, which does have both under those
// names, it resolves wherever this crate does.
#[doc(hidden)]
pub use ::quote::ToTokens as __ohelpersToTokens;
#[doc(hidden)]
pub use ::syn::token::Token as __ohelpersToken;

/// Debug logging shim over [`odebug`].
///
/// The crate previously logged through `debug_helpers::debug_file`, which no longer exists;
/// that crate is now `odebug` and its entry point is the `odebug!` macro. This preserves the
/// three call shapes the rest of the crate (and its exported macros) already use, so the call
/// sites did not have to be rewritten one by one.
///
/// - `__pmmh_debug_file!(!"fmt {}", arg)` formats, the leading `!` marks the format form
/// - `__pmmh_debug_file!(value, "label")` logs a labelled value, label first in the output
/// - `__pmmh_debug_file!("message")` logs a plain message
#[doc(hidden)]
#[macro_export]
macro_rules! __pmmh_debug_file {
    (!$fmt:expr, $($arg:tt)+) => {
        ::odebug::odebug!($fmt, $($arg)+)
    };
    (!$msg:expr) => {
        ::odebug::odebug!($msg)
    };
    ($value:expr, $label:expr) => {
        ::odebug::odebug!("{}\n{}", $label, $value)
    };
    ($msg:expr) => {
        ::odebug::odebug!($msg)
    };
}

pub mod declaration;
pub mod define;
pub mod dsl_macros;
pub mod find_first;
pub mod param;
pub mod parse_utils;
pub mod punct_set;
pub mod quote_if;
pub mod stringify;

use lazy_static::lazy_static;

pub static EMPTY: &str = "";
lazy_static! {
    pub static ref EMPTY_STR: String = EMPTY.to_string();
}

pub type TokenStream2 = proc_macro2::TokenStream;
pub type Ident2 = proc_macro2::Ident;
