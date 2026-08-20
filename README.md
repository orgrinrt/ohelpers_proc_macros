ohelpers_proc_macros
============
[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/ohelpers_proc_macros.svg)](https://github.com/orgrinrt/ohelpers_proc_macros/stargazers) 
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/ohelpers_proc_macros.svg)](https://github.com/orgrinrt/ohelpers_proc_macros/issues) 
[![Current Version](https://img.shields.io/badge/version-0.1.0-orange.svg)](https://github.com/orgrinrt/ohelpers_proc_macros) 

A collection of miscellaneous helpers, shorthands and common useful bits for Rust proc macros.

---
## Buy me a coffee

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

---

## Usage

These are used from inside another crate's proc macro, so the examples below are written the way a
macro author calls them.

`quote_if!` emits a block only when a predicate holds, which is the thing that otherwise turns every
optional fragment into an `if`/`else` wrapped around two `quote!` calls:

```rust
use ohelpers_proc_macros::{quote_if, token_name, TokenStream2};
use quote::quote;

let derive_debug = true;
let emitted: TokenStream2 = quote_if!(derive_debug, {
    #[derive(Debug)]
});
assert!(!emitted.to_string().is_empty());

// false emits nothing at all, rather than an empty block
let emitted: TokenStream2 = quote_if!(false, { #[derive(Debug)] });
assert!(emitted.to_string().is_empty());

// `some` follows an Option without unwrapping it first
let maybe: Option<TokenStream2> = Some(quote! { value });
assert_eq!(quote_if!(some maybe).to_string(), quote! { value }.to_string());
```

`token_name!` gives the last segment of a type path, and sees through a generic parameter to the type
it was instantiated with:

```rust
use ohelpers_proc_macros::token_name;

struct Widget;
assert_eq!(token_name!(ty Widget), "Widget");

fn named<T>() -> &'static str {
    token_name!(ty T)
}
assert_eq!(named::<u32>(), "u32");
assert_eq!(named::<String>(), "String");
```

Every assertion above was run against the crate rather than written from the macro names.

---

## License
>You can check out the full license [here](https://github.com/orgrinrt/ohelpers_proc_macros/blob/main/LICENSE)

This project is licensed under the terms of the **MPL-2.0** license.
