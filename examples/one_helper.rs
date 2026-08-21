//! The smallest thing this crate does: emit a fragment only when a condition holds.
//!
//! Without `quote_if!`, an optional fragment is an `if`/`else` wrapped around two `quote!`
//! calls, one of which emits nothing. That shape is fine once and unreadable by the fourth
//! time, which is what a proc macro of any size ends up with.
//!
//! ```bash
//! cargo run --example one_helper
//! ```

use ohelpers_proc_macros::{quote_if, TokenStream2};
use quote::quote;

fn main() {
    // The condition holds, so the fragment is emitted.
    let derive_debug = true;
    let emitted: TokenStream2 = quote_if!(derive_debug, {
        #[derive(Debug)]
    });
    println!("with the condition true:  {}", emitted);

    // It does not, so nothing is. Not an empty block, nothing: the difference matters when the
    // fragment is being interpolated into a larger `quote!`.
    let emitted: TokenStream2 = quote_if!(false, {
        #[derive(Debug)]
    });
    println!("with it false:            {:?}", emitted.to_string());

    // An `Option` is followed without unwrapping it first, which is the form that keeps a
    // caller from writing the same `match` around every optional piece.
    let present: Option<TokenStream2> = Some(quote! { value });
    let absent: Option<TokenStream2> = None;
    println!("some(Some(..)):           {}", quote_if!(some present));
    println!(
        "some(None):               {:?}",
        quote_if!(some absent).to_string()
    );
}
