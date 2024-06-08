use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

/// The value of each field is the index of the corresponding column.
pub(crate) const ARITH_COL_MAP: ArithCols<usize> = make_col_map();
pub(crate) const N_ARITH_COLS: usize = core::mem::size_of::<ArithCols<u8>>();
pub(crate) const N_OP_COLS: usize = core::mem::size_of::<OpCols<u8>>();

/// Flag columns for the operation to perform.
#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OpCols<T> {
    /// Addition.
    pub f_add: T,
    /// Subtraction.
    pub f_sub: T,
    /// Unsigned less than.
    pub f_ltu: T,
    /// Signed less than.
    pub f_lts: T,
    /// Unsigned greater than or equal to.
    pub f_geu: T,
    /// Signed greater than or equal to.
    pub f_ges: T,
}

/// Columns for the arithmetic stark.
#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArithCols<T> {
    /// The operation to perform.
    pub op: OpCols<T>,
    /// First operand.
    pub in0: T,
    /// Second operand.
    pub in1: T,
    /// Output.
    pub out: T,
    /// Auxiliary column.
    pub aux: T,
    /// Contains `in0 + 2^31`. Used for signed less than.
    pub in0_bias: T,
    /// Contains `in1 + 2^31`. Used for signed less than.
    pub in1_bias: T,
    /// Auxiliary column used for signed less than.
    pub in0_aux: T,
    /// Auxiliary column used for signed less than.
    pub in1_aux: T,
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

impl<T: Copy> Borrow<[T; N_ARITH_COLS]> for ArithCols<T> {
    fn borrow(&self) -> &[T; N_ARITH_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<[T; N_ARITH_COLS]> for ArithCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_ARITH_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy, I> Index<I> for ArithCols<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;
    fn index(&self, i: I) -> &Self::Output {
        let arr: &[T; N_ARITH_COLS] = self.borrow();
        <[T] as Index<I>>::index(arr, i)
    }
}

impl<T: Copy, I> IndexMut<I> for ArithCols<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        let arr: &mut [T; N_ARITH_COLS] = self.borrow_mut();
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
