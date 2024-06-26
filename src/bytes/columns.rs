use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use rizzo_derive::{Columns, DerefColumns};

use crate::bytes::BYTES_WORD;

/// The value of each struct field is the index of the corresponding column.
pub(crate) const BYTE_COL_MAP: ByteCols<usize> = make_col_map();
/// The number of field elements in `ByteCols`.
pub(crate) const N_BYTE_COLS: usize = core::mem::size_of::<ByteCols<u8>>();

const BITS_U8: usize = 8;

/// Range checking columns.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub(crate) struct RangeCheck<T> {
    /// The range check counter.
    pub count: T,
    /// The range check frequency.
    pub freq: T,
}

/// Columns for the byte packing stark.
#[repr(C)]
#[derive(Columns, DerefColumns, Clone, Debug)]
pub(crate) struct ByteCols<T> {
    /// 1 if this is a write operation, 0 for a read operation.
    pub f_rw: T,
    /// 1 if `bytes` should be interpreted as signed (lb or lh).
    pub f_signed: T,
    /// Extension byte. 0xff if `bytes` is signed and sign bit is 1, 0 otherwise.
    pub ext_byte: T,
    /// The starting virtual address. Segment is always main memory.
    pub adr_virt: T,
    /// The timestamp of the memory operation.
    pub time: T,
    /// Contains a single field element set to 1 at index `length - 1`.
    pub len_idx: [T; BYTES_WORD],
    /// The LE byte decomposition of the value being packed or unpacked.
    pub bytes: [T; BYTES_WORD],
    /// The LE bit decomposition of the most significant byte of `bytes`.
    pub high_bits: [T; BITS_U8],
    /// Range check columns.
    pub range_check: RangeCheck<T>,
}

impl<T: Copy> ByteCols<T> {
    pub(crate) fn to_vec(&self) -> Vec<T> {
        Borrow::<[T; N_BYTE_COLS]>::borrow(self).to_vec()
    }
}

const fn make_col_map() -> ByteCols<usize> {
    let arr = crate::util::indices_arr::<N_BYTE_COLS>();
    unsafe { core::mem::transmute::<[usize; N_BYTE_COLS], ByteCols<usize>>(arr) }
}
