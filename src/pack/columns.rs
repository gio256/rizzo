use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use crate::pack::N_BYTES;

const N_BITS: usize = 8;
pub(crate) const N_PACK_COLS: usize = core::mem::size_of::<PackCols<u8>>();
pub(crate) const PACK_COL_MAP: PackCols<usize> = make_col_map();

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
pub(crate) struct PackCols<T> {
    /// 1 if this is a write operation, 0 for a read operation.
    pub f_rw: T,
    /// 1 if `bytes` should be interpreted as signed (lb or lh).
    pub f_signed: T,
    /// 1 if `bytes` should be sign extended and the sign bit is 1.
    pub f_sign_ext: T,
    /// The starting virtual address of the memory operation.
    pub adr_virt: T,
    /// The timestamp of the memory operation.
    pub time: T,
    /// Contains a single field element set to 1 at index `length - 1`.
    pub len_idx: [T; N_BYTES],
    /// The LE byte decomposition of the value being packed or unpacked.
    pub bytes: [T; N_BYTES],
    /// The LE bit decomposition of the most significant byte of `bytes`.
    pub high_bits: [T; N_BITS],
    /// Range check columns.
    pub range_check: RangeCheck<T>,
}

impl<T: Copy> PackCols<T> {
    pub(crate) fn to_vec(&self) -> Vec<T> {
        Borrow::<[T; N_PACK_COLS]>::borrow(self).to_vec()
    }
}

const fn make_col_map() -> PackCols<usize> {
    let arr = crate::util::indices_arr::<N_PACK_COLS>();
    unsafe { core::mem::transmute::<[usize; N_PACK_COLS], PackCols<usize>>(arr) }
}
impl<T: Copy> Borrow<PackCols<T>> for [T; N_PACK_COLS] {
    fn borrow(&self) -> &PackCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<PackCols<T>> for [T; N_PACK_COLS] {
    fn borrow_mut(&mut self) -> &mut PackCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> Borrow<[T; N_PACK_COLS]> for PackCols<T> {
    fn borrow(&self) -> &[T; N_PACK_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<[T; N_PACK_COLS]> for PackCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_PACK_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy, I> Index<I> for PackCols<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;
    fn index(&self, i: I) -> &Self::Output {
        let arr: &[T; N_PACK_COLS] = self.borrow();
        <[T] as Index<I>>::index(arr, i)
    }
}
impl<T: Copy, I> IndexMut<I> for PackCols<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        let arr: &mut [T; N_PACK_COLS] = self.borrow_mut();
        <[T] as IndexMut<I>>::index_mut(arr, i)
    }
}

impl<T: Copy> Deref for PackCols<T> {
    type Target = [T; N_PACK_COLS];
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> DerefMut for PackCols<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
