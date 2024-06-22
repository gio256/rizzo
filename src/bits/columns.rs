use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use rizzo_derive::{DerefColumns, Columns};

pub(crate) const WORD_BITS: usize = 32;

/// The value of each struct field is the index of the corresponding column.
pub(crate) const BIT_COL_MAP: BitCols<usize> = make_col_map();
/// The value of each struct field is the index of the corresponding column.
pub(crate) const OP_COL_MAP: OpCols<usize> = make_op_col_map();
/// The number of field elements in `BitCols`.
pub(crate) const N_BIT_COLS: usize = core::mem::size_of::<BitCols<u8>>();
/// The number of field elements in `OpCols`.
pub(crate) const N_OP_COLS: usize = core::mem::size_of::<OpCols<u8>>();

/// Flag columns for the operation to perform.
#[repr(C)]
#[derive(DerefColumns, Clone, Debug, Default)]
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
#[derive(Columns, Clone, Debug)]
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
