use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

pub const N_MEM_CHANNELS: usize = 3;

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct MemChannel<T> {
    pub f_on: T,
    pub f_rw: T,
    pub adr_seg: T,
    pub adr_virt: T,
    pub val: T,
}

pub const N_MEM_CHAN_COLS: usize = core::mem::size_of::<MemChannel<u8>>();

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct OpCols<T> {
    pub f_alu: T,
    pub f_lb: T,
    pub f_jal: T,
    pub f_jalr: T,
    pub f_beq: T,
    pub f_bne: T,
    pub f_blt: T,
    pub f_bge: T,
    pub f_bltu: T,
    pub f_bgeu: T,
    pub f_lui: T,
}

pub const N_OP_COLS: usize = core::mem::size_of::<OpCols<u8>>();

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CpuCols<T> {
    pub clock: T,
    pub pc: T,
    pub op: OpCols<T>,
    // pub op1: T,
    // pub op2: T,
    // pub op_w: T,
    // pub imm1: T,
    // pub imm2: T,
    pub rs1: T,
    pub rs2: T,
    pub rd: T,
    pub imm: T,
    // pub rs1_val: MemChannel<T>,
    // pub rs2_val: MemChannel<T>,
    // pub rd_val: MemChannel<T>,
    pub membus: [MemChannel<T>; N_MEM_CHANNELS],
}

pub const N_CPU_COLS: usize = core::mem::size_of::<CpuCols<u8>>();
pub const CPU_COL_MAP: CpuCols<usize> = make_cpu_col_map();

impl<T: Copy> Borrow<CpuCols<T>> for [T; N_CPU_COLS] {
    fn borrow(&self) -> &CpuCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<CpuCols<T>> for [T; N_CPU_COLS] {
    fn borrow_mut(&mut self) -> &mut CpuCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> Borrow<[T; N_CPU_COLS]> for CpuCols<T> {
    fn borrow(&self) -> &[T; N_CPU_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<[T; N_CPU_COLS]> for CpuCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_CPU_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy, I> Index<I> for CpuCols<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;
    fn index(&self, i: I) -> &Self::Output {
        let arr: &[T; N_CPU_COLS] = self.borrow();
        <[T] as Index<I>>::index(arr, i)
    }
}
impl<T: Copy, I> IndexMut<I> for CpuCols<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, i: I) -> &mut Self::Output {
        let arr: &mut [T; N_CPU_COLS] = self.borrow_mut();
        <[T] as IndexMut<I>>::index_mut(arr, i)
    }
}
const fn make_cpu_col_map() -> CpuCols<usize> {
    let arr = crate::util::indices_arr::<N_CPU_COLS>();
    unsafe { core::mem::transmute::<[usize; N_CPU_COLS], CpuCols<usize>>(arr) }
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
