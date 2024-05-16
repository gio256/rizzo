use core::borrow::{Borrow, BorrowMut};

pub const LIMB_BITS: usize = 8;
pub const WORD_BITS: usize = 32;
pub const N_LIMBS: usize = n_limbs();

const fn n_limbs() -> usize {
    assert!(
        WORD_BITS % LIMB_BITS == 0,
        "LIMB_BITS must divide WORD_BITS"
    );
    let n = WORD_BITS / LIMB_BITS;
    assert!(n % 2 == 0, "N_LIMBS must be even");
    n
}

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Word<T>([T; N_LIMBS]);

impl<T: Copy> Borrow<Word<T>> for [T; N_LIMBS] {
    fn borrow(&self) -> &Word<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<Word<T>> for [T; N_LIMBS] {
    fn borrow_mut(&mut self) -> &mut Word<T> {
        unsafe { core::mem::transmute(self) }
    }
}
