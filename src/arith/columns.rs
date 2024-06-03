use core::borrow::{Borrow, BorrowMut};

pub(crate) const N_ARITH_COLS: usize = core::mem::size_of::<ArithCols<u8>>();
pub(crate) const ARITH_COL_MAP: ArithCols<usize> = make_col_map();

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OpCols<T> {
    pub f_add: T,
    pub f_sub: T,
    pub f_ltu: T,
}

#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArithCols<T> {
    pub op: OpCols<T>,
    pub in0: T,
    pub in1: T,
    pub out: T,
    pub aux: T,
}

const fn make_col_map() -> ArithCols<usize> {
    let arr = crate::util::indices_arr::<N_ARITH_COLS>();
    unsafe { core::mem::transmute::<[usize; N_ARITH_COLS], ArithCols<usize>>(arr) }
}
impl<T: Copy> Borrow<ArithCols<T>> for [T; N_ARITH_COLS] {
    fn borrow(&self) -> &ArithCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<ArithCols<T>> for [T; N_ARITH_COLS] {
    fn borrow_mut(&mut self) -> &mut ArithCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
