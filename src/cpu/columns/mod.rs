use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use static_assertions::const_assert;

mod shared;
use shared::SharedCols;

/// The total number of memory channels.
pub(crate) const N_MEM_CHANNELS: usize = 3;
/// The number of field elements in `MemChannel`.
pub(crate) const N_MEM_CHANNEL_COLS: usize = core::mem::size_of::<MemChannel<u8>>();

/// Columns for a single memory channel.
#[repr(C)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct MemChannel<T> {
    /// 1 if this memory channel is used.
    pub f_on: T,
    /// 1 if this is a write operation, 0 for a read operation
    pub f_rw: T,
    /// Address segment (register or main memory).
    pub adr_seg: T,
    /// Virtual address.
    pub adr_virt: T,
    /// The value in the memory channel.
    pub val: T,
}

/// The number of field elements in `OpCols`.
pub(crate) const N_OP_COLS: usize = core::mem::size_of::<OpCols<u8>>();

/// Flag columns for the operation to perform.
#[repr(C)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct OpCols<T> {
    pub f_arith: T,
    pub f_bits: T,
    pub f_lw: T,
    pub f_lh: T,
    pub f_lb: T,
    pub f_lhu: T,
    pub f_lbu: T,
    pub f_sw: T,
    pub f_sh: T,
    pub f_sb: T,
    pub f_jal: T,
    pub f_jalr: T,
    pub f_beq: T,
    pub f_bne: T,
    pub f_bltu: T,
    pub f_bgeu: T,
    pub f_blt: T,
    pub f_bge: T,
}

/// The value of each struct field is the index of the corresponding column.
pub(crate) const CPU_COL_MAP: CpuCols<usize> = make_col_map();
/// The number of field elements in `CpuCols`.
pub(crate) const N_CPU_COLS: usize = core::mem::size_of::<CpuCols<u8>>();

/// Columns for the cpu stark.
#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct CpuCols<T: Copy> {
    /// CPU clock.
    pub clock: T,
    /// Program counter.
    pub pc: T,
    /// The operation to perform.
    pub op: OpCols<T>,
    /// The opcode value (our internal `Opcode` enum).
    pub opcode: T,
    /// Source register `rs1`.
    pub rs1: T,
    /// Source register `rs2`.
    pub rs2: T,
    /// Destination register `rd`.
    pub rd: T,
    /// The immediate value.
    pub imm: T,
    /// 1 if the immediate value should be used.
    pub f_imm: T,
    /// Auxiliary column.
    pub f_aux0: T,
    /// Auxiliary column.
    pub f_aux1: T,
    /// Memory channels.
    pub membus: [MemChannel<T>; N_MEM_CHANNELS],
    /// Columns dependent on the operation.
    pub shared: SharedCols<T>,
}

impl<T: Copy> CpuCols<T> {
    /// Returns the memory channel associated with register `rd`.
    pub(crate) fn rd_channel(&self) -> &MemChannel<T> {
        const_assert!(N_MEM_CHANNELS > 0);
        &self.membus[0]
    }

    /// Returns the memory channel associated with register `rs1`.
    pub(crate) fn rs1_channel(&self) -> &MemChannel<T> {
        const_assert!(N_MEM_CHANNELS > 1);
        &self.membus[1]
    }

    /// Returns the memory channel associated with register `rs2`.
    pub(crate) fn rs2_channel(&self) -> &MemChannel<T> {
        const_assert!(N_MEM_CHANNELS > 2);
        &self.membus[2]
    }
}

const fn make_col_map() -> CpuCols<usize> {
    let arr = crate::util::indices_arr::<N_CPU_COLS>();
    unsafe { core::mem::transmute::<[usize; N_CPU_COLS], CpuCols<usize>>(arr) }
}

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
