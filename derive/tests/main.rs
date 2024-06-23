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
struct TestColumns<T: Copy> {
    pub felt: T,
    pub felt_arr: [T; N_VALS],
    pub sub: SubColumns<T>,
}
