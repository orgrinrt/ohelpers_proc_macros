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
