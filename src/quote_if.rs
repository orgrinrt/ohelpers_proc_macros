#[macro_export]
macro_rules! quote_if {
    ($predicate:expr, { $($quote:tt)* }) => {
        if $predicate { quote!{
            $($quote)*
        }} else {
            quote!()
        };
    };
    ($predicate:expr, $quote:ident) => {
        if $predicate { quote!{
            #$quote
        }} else {
            quote!()
        };
    };
    (some $quote:ident) => {
        if $quote.is_some() { quote!{
            #$quote
        }} else {
            quote!()
        };
    };
}

#[macro_export]
macro_rules! format_ident_if {
    ($predicate:expr, $fmt:expr, $($param:ident)+) => {
        if $predicate {
            format_ident!($fmt, $($param)+)
        } else {
            format_ident!("{}", $($param)+)
        };
    };
}
