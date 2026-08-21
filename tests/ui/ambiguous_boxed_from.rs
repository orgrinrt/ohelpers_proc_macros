//! `StringableParsable::from` on a box is ambiguous, and deliberately so.
//!
//! Two impls apply: the blanket one wraps the box, and `From<Box<T>>` unwraps it. A caller
//! says which by naming the type. The ambiguity is the cost of having both conversions, and
//! it is pinned here because the last attempt to remove it removed the unwrapping one, which
//! turned a conversion into a silently different type for anyone using `.into()`.

use ohelpers_proc_macros::stringify::StringableParsable;
use quote::quote;

fn main() {
    let _ambiguous = StringableParsable::from(Box::new(quote! { a + b }));
}
