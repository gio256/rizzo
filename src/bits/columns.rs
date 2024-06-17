use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use plonky2::field::packed::PackedField;
use static_assertions::const_assert;

pub(crate) const WORD_BITS: usize = 32;

/// The value of each field is the index of the corresponding column.
pub(crate) const BIT_COL_MAP: BitCols<usize> = make_col_map();
pub(crate) const N_BIT_COLS: usize = core::mem::size_of::<BitCols<u8>>();
pub(crate) const OP_COL_MAP: OpCols<usize> = make_op_col_map();
pub(crate) const N_OP_COLS: usize = core::mem::size_of::<OpCols<u8>>();

/// Flag columns for the operation to perform.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub(crate) struct OpCols<T: Copy> {
    pub f_and: T,
    pub f_xor: T,
    pub f_or: T,
    pub f_sll: T,
    pub f_srl: T,
    pub f_sra: T,
}

/// Columns for the bit stark.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub(crate) struct BitCols<T: Copy> {
    /// The operation to perform.
    pub op: OpCols<T>,
    /// First operand, decomposed into bits.
    pub in0: [T; WORD_BITS],
    /// Second operand, decomposed into bits.
    pub in1: [T; WORD_BITS],
    /// Output, stored as a single field element.
    pub out: T,
    /// `in0 & in1`, stored as a single field element.
    pub and: T,
}

impl<T: Copy> BitCols<T> {
    pub(crate) fn to_vec(&self) -> Vec<T> {
        Borrow::<[T; N_BIT_COLS]>::borrow(self).to_vec()
    }
}

const fn make_col_map() -> BitCols<usize> {
    let arr = crate::util::indices_arr::<N_BIT_COLS>();
    unsafe { core::mem::transmute::<[usize; N_BIT_COLS], BitCols<usize>>(arr) }
}

const fn make_op_col_map() -> OpCols<usize> {
    let arr = crate::util::indices_arr::<N_OP_COLS>();
    unsafe { core::mem::transmute::<[usize; N_OP_COLS], OpCols<usize>>(arr) }
}

impl<T: Copy> Borrow<BitCols<T>> for [T; N_BIT_COLS] {
    fn borrow(&self) -> &BitCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<BitCols<T>> for [T; N_BIT_COLS] {
    fn borrow_mut(&mut self) -> &mut BitCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> Borrow<[T; N_BIT_COLS]> for BitCols<T> {
    fn borrow(&self) -> &[T; N_BIT_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<[T; N_BIT_COLS]> for BitCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_BIT_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy, I> Index<I> for BitCols<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;
    fn index(&self, i: I) -> &Self::Output {
        let arr: &[T; N_BIT_COLS] = self.borrow();
        <[T] as Index<I>>::index(arr, i)
    }
}

impl<T: Copy, I> IndexMut<I> for BitCols<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        let arr: &mut [T; N_BIT_COLS] = self.borrow_mut();
        <[T] as IndexMut<I>>::index_mut(arr, i)
    }
}

impl<T: Copy> Deref for OpCols<T> {
    type Target = [T; N_OP_COLS];
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> DerefMut for OpCols<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
