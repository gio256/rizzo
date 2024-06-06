use core::borrow::{Borrow, BorrowMut};
use core::fmt::{Debug, Formatter};

pub(crate) const N_SHARED_COLS: usize = core::mem::size_of::<SharedCols<u8>>();

/// Columns intended to be shared, but currently only used by branching ops.
#[derive(Clone, Copy)]
pub(crate) union SharedCols<T: Copy> {
    branch: BranchCols<T>,
}

impl<T: Copy> SharedCols<T> {
    pub(crate) fn branch(&self) -> &BranchCols<T> {
        unsafe { &self.branch }
    }
    pub(crate) fn branch_mut(&mut self) -> &mut BranchCols<T> {
        unsafe { &mut self.branch }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BranchCols<T> {
    pub f_take_branch: T,
    pub diff_pinv: T,
}

impl<T: Copy + Debug> Debug for SharedCols<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let arr: &[T; N_SHARED_COLS] = self.borrow();
        Debug::fmt(arr, f)
    }
}

impl<T: Copy> Borrow<SharedCols<T>> for [T; N_SHARED_COLS] {
    fn borrow(&self) -> &SharedCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<SharedCols<T>> for [T; N_SHARED_COLS] {
    fn borrow_mut(&mut self) -> &mut SharedCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> Borrow<[T; N_SHARED_COLS]> for SharedCols<T> {
    fn borrow(&self) -> &[T; N_SHARED_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

impl<T: Copy> BorrowMut<[T; N_SHARED_COLS]> for SharedCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_SHARED_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
