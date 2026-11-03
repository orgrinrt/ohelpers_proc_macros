# `ohelpers_proc_macros`

> Miscellaneous helpers and shorthands for writing rust proc macros.

A small collection of helpers that a macro author calls from inside a proc macro. `quote_if!` emits a fragment only when a predicate holds, instead of an `if`/`else` wrapped around two `quote!` calls, and `token_name!` gives the last segment of a type path, seeing through a generic parameter to the type it was instantiated with.

## Usage

Not published to crates.io yet, so it is a git dependency for now.

```toml
[dependencies]
ohelpers_proc_macros = { git = "https://github.com/orgrinrt/ohelpers_proc_macros", branch = "dev" }
```

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

`examples/one_helper.rs` and `examples/every_helper.rs` run, and `cargo test` executes each and checks what it printed.

## Features

`debug_file` writes what the macros expand to into a file through `odebug`, which is how to find out what a generated body is. `no_std` and `no_alloc` say that the consumer is `#![no_std]` or has no allocator, and they change nothing in what this crate emits, since the exported macros reach `Box` and `ToString` through re-exports and not by naming `std` or `alloc`. `tests/feature_matrix.rs` compiles a real consumer under each. This crate itself is always `std`, as it runs inside the compiler.

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/ohelpers_proc_macros/blob/dev/LICENSE)
