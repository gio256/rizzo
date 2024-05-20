use core::borrow::{Borrow, BorrowMut};

pub(crate) const N_ALU_COLS: usize = core::mem::size_of::<AluCols<u8>>();
pub(crate) const ALU_COL_MAP: AluCols<usize> = make_col_map();

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AluOpCols<T> {
    pub f_add: T,
    pub f_sub: T,
    pub f_lt: T,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AluCols<T> {
    pub op: AluOpCols<T>,
    pub in0: T,
    pub in1: T,
    pub out: T,
    pub aux: T,
}

const fn make_col_map() -> AluCols<usize> {
    let arr = crate::util::indices_arr::<N_ALU_COLS>();
    unsafe { core::mem::transmute::<[usize; N_ALU_COLS], AluCols<usize>>(arr) }
}
impl<T: Copy> Borrow<AluCols<T>> for [T; N_ALU_COLS] {
    fn borrow(&self) -> &AluCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<AluCols<T>> for [T; N_ALU_COLS] {
    fn borrow_mut(&mut self) -> &mut AluCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
