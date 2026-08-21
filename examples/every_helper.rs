//! Every helper this crate exports, and what each one saves writing.
//!
//! These are called from inside another crate's procedural macro, so the example is written
//! the way a macro author calls them rather than the way a consumer of that macro would.
//!
//! ```bash
//! cargo run --example every_helper
//! ```

use ohelpers_proc_macros::{quote_if, token_name, TokenStream2};
use quote::quote;

struct Widget;

/// A type's name from inside a generic, which is where it is not otherwise available: the
/// source says `T` and what a macro needs is what `T` turned out to be.
fn named<T>() -> &'static str {
    token_name!(ty T)
}

fn main() {
    println!("== quote_if, the conditional fragment");
    for condition in [true, false] {
        let emitted: TokenStream2 = quote_if!(condition, {
            #[derive(Debug)]
        });
        println!("  {condition:<5} -> {:?}", emitted.to_string());
    }

    println!();
    println!("== quote_if some, following an Option");
    let present: Option<TokenStream2> = Some(quote! { value });
    let absent: Option<TokenStream2> = None;
    println!("  Some -> {:?}", quote_if!(some present).to_string());
    println!("  None -> {:?}", quote_if!(some absent).to_string());

    println!();
    println!("== token_name, the last segment of a path");
    println!("  a concrete type:   {}", token_name!(ty Widget));
    println!("  through a generic: {}", named::<u32>());
    println!("  and another:       {}", named::<String>());
}
