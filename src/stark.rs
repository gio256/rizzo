use core::ops::Deref;

use plonky2::field::extension::Extendable;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use starky::config::StarkConfig;
use starky::cross_table_lookup::{CrossTableLookup, TableIdx, TableWithColumns};
use starky::evaluation_frame::StarkFrame;
use starky::stark::Stark;

use crate::cpu::columns::N_MEM_CHANNELS;
use crate::pack::N_BYTES;
use crate::{arith, cpu, logic, mem, pack};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Table {
    Arith,
    Logic,
    Cpu,
    Mem,
    Pack,
}

fn all_cross_table_lookups<F: Field>() -> Vec<CrossTableLookup<F>> {
    vec![ctl_arith(), ctl_logic(), ctl_byte_packing(), ctl_mem()]
}

fn ctl_arith<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![
        cpu::stark::ctl_looking_arith_reg(),
        cpu::stark::ctl_looking_arith_imm(),
        cpu::stark::ctl_looking_branch(),
    ];
    let looked = arith::stark::ctl_looked();
    CrossTableLookup::new(looking, looked)
}

fn ctl_logic<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![
        cpu::stark::ctl_looking_logic_reg(),
        cpu::stark::ctl_looking_logic_imm(),
    ];
    let looked = logic::stark::ctl_looked();
    CrossTableLookup::new(looking, looked)
}

fn ctl_byte_packing<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![
        cpu::stark::ctl_looking_pack(),
        cpu::stark::ctl_looking_unpack(),
    ];
    let looked = pack::stark::ctl_looked();
    CrossTableLookup::new(looking, looked)
}

fn ctl_mem<F: Field>() -> CrossTableLookup<F> {
    let cpu = (0..N_MEM_CHANNELS).map(cpu::stark::ctl_looking_mem);
    let pack = (0..N_BYTES).map(pack::stark::ctl_looking_mem);
    let looking = cpu.chain(pack).collect();
    let looked = mem::stark::ctl_looked();
    CrossTableLookup::new(looking, looked)
}
