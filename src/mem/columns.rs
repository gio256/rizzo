use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

/// Range checking columns.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub(crate) struct RangeCheck<T> {
    /// The value to range check.
    pub val: T,
    /// The range check counter.
    pub count: T,
    /// The range check frequency.
    pub freq: T,
}

/// The value of each struct field is the index of the corresponding column.
pub(crate) const MEM_COL_MAP: MemCols<usize> = make_col_map();
/// The number of field elements in `MemCols`.
pub(crate) const N_MEM_COLS: usize = core::mem::size_of::<MemCols<u8>>();

/// Columns for the memory stark.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub(crate) struct MemCols<T> {
    /// 1 if this is a real memory operation, 0 if it's a padding row.
    pub f_on: T,
    /// 1 if this is a write operation, 0 for a read operation.
    pub f_rw: T,
    /// Timestamp.
    pub time: T,
    /// Address segment (register or main memory).
    pub adr_seg: T,
    /// Virtual address.
    pub adr_virt: T,
    /// 32-bit memory value. Registers will use all 4 bytes while main memory
    /// will use 1 byte.
    pub val: T,
    /// Contains `(1 - f_seg_diff - f_virt_diff) * (1 - f_reg0)`.
    pub aux: T,
    /// 1 if this operation targets register `x0`.
    pub f_reg0: T,
    /// 1 if `adr_seg` differs in the next row.
    pub f_seg_diff: T,
    /// 1 if `adr_virt` differs in the next row and `adr_seg` does not.
    pub f_virt_diff: T,
    /// Range check columns.
    pub range_check: RangeCheck<T>,
}

impl<T: Copy> MemCols<T> {
    pub(crate) fn to_vec(&self) -> Vec<T> {
        Borrow::<[T; N_MEM_COLS]>::borrow(self).to_vec()
    }
}

const fn make_col_map() -> MemCols<usize> {
    let arr = crate::util::indices_arr::<N_MEM_COLS>();
    unsafe { core::mem::transmute::<[usize; N_MEM_COLS], MemCols<usize>>(arr) }
}

impl<T: Copy> Borrow<MemCols<T>> for [T; N_MEM_COLS] {
    fn borrow(&self) -> &MemCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<MemCols<T>> for [T; N_MEM_COLS] {
    fn borrow_mut(&mut self) -> &mut MemCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> Borrow<[T; N_MEM_COLS]> for MemCols<T> {
    fn borrow(&self) -> &[T; N_MEM_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<[T; N_MEM_COLS]> for MemCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_MEM_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy, I> Index<I> for MemCols<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;
    fn index(&self, i: I) -> &Self::Output {
        let arr: &[T; N_MEM_COLS] = self.borrow();
        <[T] as Index<I>>::index(arr, i)
    }
}

impl<T: Copy, I> IndexMut<I> for MemCols<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        let arr: &mut [T; N_MEM_COLS] = self.borrow_mut();
        <[T] as IndexMut<I>>::index_mut(arr, i)
    }
}

impl<T: Copy> Deref for MemCols<T> {
    type Target = [T; N_MEM_COLS];
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> DerefMut for MemCols<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
