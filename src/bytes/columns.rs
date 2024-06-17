use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

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
#[derive(Clone, Debug, Default)]
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

impl<T: Copy> Borrow<ByteCols<T>> for [T; N_BYTE_COLS] {
    fn borrow(&self) -> &ByteCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<ByteCols<T>> for [T; N_BYTE_COLS] {
    fn borrow_mut(&mut self) -> &mut ByteCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> Borrow<[T; N_BYTE_COLS]> for ByteCols<T> {
    fn borrow(&self) -> &[T; N_BYTE_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<[T; N_BYTE_COLS]> for ByteCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_BYTE_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy, I> Index<I> for ByteCols<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;
    fn index(&self, i: I) -> &Self::Output {
        let arr: &[T; N_BYTE_COLS] = self.borrow();
        <[T] as Index<I>>::index(arr, i)
    }
}

impl<T: Copy, I> IndexMut<I> for ByteCols<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        let arr: &mut [T; N_BYTE_COLS] = self.borrow_mut();
        <[T] as IndexMut<I>>::index_mut(arr, i)
    }
}

impl<T: Copy> Deref for ByteCols<T> {
    type Target = [T; N_BYTE_COLS];
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> DerefMut for ByteCols<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
