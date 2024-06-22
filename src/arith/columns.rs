use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use rizzo_derive::{DerefColumns, Columns};

/// The value of each struct field is the index of the corresponding column.
pub(crate) const ARITH_COL_MAP: ArithCols<usize> = make_col_map();
/// The value of each struct field is the index of the corresponding column.
pub(crate) const OP_COL_MAP: OpCols<usize> = make_op_col_map();
/// The number of field elements in `ArithCols`.
pub(crate) const N_ARITH_COLS: usize = core::mem::size_of::<ArithCols<u8>>();
/// The number of field elements in `OpCols`.
pub(crate) const N_OP_COLS: usize = core::mem::size_of::<OpCols<u8>>();

/// Flag columns for the operation to perform.
#[repr(C)]
#[derive(DerefColumns, Clone, Debug, Default)]
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
#[derive(Columns, Clone, Debug)]
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

impl<T: Copy> ArithCols<T> {
    pub(crate) fn to_vec(&self) -> Vec<T> {
        Borrow::<[T; N_ARITH_COLS]>::borrow(self).to_vec()
    }
}

const fn make_col_map() -> ArithCols<usize> {
    let arr = crate::util::indices_arr::<N_ARITH_COLS>();
    unsafe { core::mem::transmute::<[usize; N_ARITH_COLS], ArithCols<usize>>(arr) }
}

const fn make_op_col_map() -> OpCols<usize> {
    let arr = crate::util::indices_arr::<N_OP_COLS>();
    unsafe { core::mem::transmute::<[usize; N_OP_COLS], OpCols<usize>>(arr) }
}
