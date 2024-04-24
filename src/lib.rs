pub use debug_helpers::debug_file as __pmmh_debug_file;

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
