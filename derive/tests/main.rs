#![allow(unused)]
use rizzo_derive::Columns;

const N_VALS: usize = 5;

#[repr(C)]
#[derive(Columns)]
struct TestColumns<'a, T: Copy> {
    pub felt0: T,
    pub felt1: T,
    pub felt_arr: [T; N_VALS],
    _unused: core::marker::PhantomData<&'a usize>,
}
