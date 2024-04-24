#![feature(macro_metavar_expr)]

#[macro_export]
macro_rules! tuple_pat {
    ($pat:pat) => {$pat};
    ($($pat:pat),*) => { ($($pat,)*) };
}

#[macro_export]
macro_rules! params_tuple {
    ($x:ty) => (($x));
    ($($x:ty),+) => (($($x),+));
}

#[macro_export]
macro_rules! impl_opt_multiparam_trait {
    ($tr:tt, $method:ident, $ty:tt ( $($name:ident : $param:ty),* $(,)? ) { $($body:tt)* })
    => {
        impl $tr for $ty {
            type Params = $crate::params_tuple!($($param),*);

            fn $method(params: Self::Params) -> Self {
                let $crate::tuple_pat!($($name)*) = params.clone();

                $($body)*
            }
        }
    };
    ($tr:tt, $method:ident,  $ty:tt () { $($body:tt)* }) => {
        impl $tr for $ty {
            type Params = ();

            fn $method(_: Self::Params) -> Self {
                $($body)*
            }
        }
    };
    ($tr:tt, $method:ident,  $ty:tt { $($body:tt)* }) => {
        impl $tr for $ty {
            type Params = ();

            fn $method(_: Self::Params) -> Self {
                $($body)*
            }
        }
    };
}
