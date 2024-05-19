use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use crate::word::Word;

pub(crate) const N_MEM_COLS: usize = core::mem::size_of::<MemCols<u8>>();
pub(crate) const MEM_COL_MAP: MemCols<usize> = make_col_map();

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) struct MemCols<T> {
    pub f_on: T,
    pub f_rw: T,
    pub time: T,
    pub adr_seg: T,
    pub adr_virt: T,
    pub val: T,

    pub f_seg_fst_diff: T,
    pub f_virt_fst_diff: T,
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
