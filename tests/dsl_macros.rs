//------------------------------------------------------------------------------
// Copyright (c) 2025                 orgrinrt           orgrinrt@ikiuni.dev
//                                    Hiisi Digital Oy   contact@hiisi.digital
//------------------------------------------------------------------------------

//! The declaration macros, every arm of each.

use ohelpers_proc_macros::{impl_opt_multiparam_trait, params_tuple, tuple_pat};

#[test]
fn tuple_pat_binds_a_single_pattern_without_wrapping_it() {
    let tuple_pat!(single) = 7;
    assert_eq!(single, 7);
}

#[test]
fn tuple_pat_binds_several_patterns_as_a_tuple() {
    let tuple_pat!(a, b, c) = (1, 2, 3);
    assert_eq!((a, b, c), (1, 2, 3));
}

#[test]
fn params_tuple_names_a_single_type_bare() {
    let value: params_tuple!(u8) = 3;
    assert_eq!(value, 3u8);
}

#[test]
fn params_tuple_names_several_types_as_a_tuple() {
    let value: params_tuple!(u8, bool) = (3, true);
    assert_eq!(value, (3u8, true));
}

trait Build {
    type Params: Clone;
    fn build(params: Self::Params) -> Self;
}

#[derive(Debug, PartialEq)]
struct Zero;
#[derive(Debug, PartialEq)]
struct One(u8);
#[derive(Debug, PartialEq)]
struct Two(u8, bool);

// The no-parens arm.
impl_opt_multiparam_trait!(Build, build, Zero { Zero });

// The empty-parens arm.
impl_opt_multiparam_trait!(Build, build, One () { One(0) });

// The parameterised arm, at one parameter.
struct Single(u8);
impl_opt_multiparam_trait!(Build, build, Single (a: u8) { Single(a) });

// The parameterised arm, at more than one. This is the case the macro is named for.
impl_opt_multiparam_trait!(Build, build, Two (a: u8, b: bool) { Two(a, b) });

#[test]
fn arm_without_parens_takes_no_params() {
    assert_eq!(Zero::build(()), Zero);
}

#[test]
fn arm_with_empty_parens_takes_no_params() {
    assert_eq!(One::build(()), One(0));
}

#[test]
fn arm_with_one_param_binds_it() {
    assert_eq!(Single::build(5).0, 5);
}

#[test]
fn arm_with_several_params_binds_each_of_them() {
    assert_eq!(Two::build((5, true)), Two(5, true));
}
