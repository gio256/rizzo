use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use plonky2::field::packed::PackedField;
use static_assertions::const_assert;

const N_BITS: usize = 32;

/// The value of each field is the index of the respective column.
pub(crate) const LOGIC_COL_MAP: LogicCols<usize> = make_col_map();
pub(crate) const N_LOGIC_COLS: usize = core::mem::size_of::<LogicCols<u8>>();

/// Flag columns for the operation to perform.
#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct OpCols<T: Copy> {
    pub f_and: T,
    pub f_xor: T,
    pub f_or: T,
}

/// Columns for the logic stark.
#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct LogicCols<T: Copy> {
    /// The operation to perform.
    pub op: OpCols<T>,
    /// First operand, decomposed into bits.
    pub in0: [T; N_BITS],
    /// Second operand, decomposed into bits.
    pub in1: [T; N_BITS],
    /// Output, stored as a single value.
    pub out: T,
}

const fn make_col_map() -> LogicCols<usize> {
    let arr = crate::util::indices_arr::<N_LOGIC_COLS>();
    unsafe { core::mem::transmute::<[usize; N_LOGIC_COLS], LogicCols<usize>>(arr) }
}

impl<T: Copy> Borrow<LogicCols<T>> for [T; N_LOGIC_COLS] {
    fn borrow(&self) -> &LogicCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<LogicCols<T>> for [T; N_LOGIC_COLS] {
    fn borrow_mut(&mut self) -> &mut LogicCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> Borrow<[T; N_LOGIC_COLS]> for LogicCols<T> {
    fn borrow(&self) -> &[T; N_LOGIC_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<[T; N_LOGIC_COLS]> for LogicCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_LOGIC_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy, I> Index<I> for LogicCols<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;
    fn index(&self, i: I) -> &Self::Output {
        let arr: &[T; N_LOGIC_COLS] = self.borrow();
        <[T] as Index<I>>::index(arr, i)
    }
}

impl<T: Copy, I> IndexMut<I> for LogicCols<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        let arr: &mut [T; N_LOGIC_COLS] = self.borrow_mut();
        <[T] as IndexMut<I>>::index_mut(arr, i)
    }
}
