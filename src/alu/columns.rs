use core::borrow::{Borrow, BorrowMut};

pub const N_ALU_COLS: usize = core::mem::size_of::<AluCols<u8>>();

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AluCols<T> {
    pub f_add: T,
    pub f_sub: T,
    pub f_lt: T,
    pub in0: T,
    pub in1: T,
    pub out: T,
    pub aux: T,
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
