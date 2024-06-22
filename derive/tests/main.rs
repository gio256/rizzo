#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
use rizzo_derive::{Columns, DerefColumns};

const N_VALS: usize = 5;

#[repr(C)]
#[derive(DerefColumns)]
struct SubColumns<T> {
    pub felt: T,
    pub felt_arr: [T; N_VALS],
}

#[repr(C)]
#[derive(Columns)]
struct TestColumns<'a, T: Copy> {
    pub felt: T,
    pub felt_arr: [T; N_VALS],
    pub sub: SubColumns<T>,
    _unused: core::marker::PhantomData<&'a usize>,
}

#[repr(C)]
#[derive(Columns)]
struct TestColumnsConst<'a, T: Copy, const N: usize> {
    pub felt: T,
    pub felt_arr: [T; N],
    _unused: core::marker::PhantomData<&'a usize>,
}
