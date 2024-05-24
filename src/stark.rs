use core::ops::Deref;

use plonky2::field::extension::Extendable;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use starky::config::StarkConfig;
use starky::cross_table_lookup::{CrossTableLookup, TableIdx, TableWithColumns};
use starky::evaluation_frame::StarkFrame;
use starky::stark::Stark;

use crate::cpu::columns::N_MEM_CHANNELS;
use crate::{arith, cpu, mem};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Table {
    Arith,
    Cpu,
    Mem,
}

fn all_cross_table_lookups<F: Field>() -> Vec<CrossTableLookup<F>> {
    vec![ctl_arith(), ctl_mem()]
}

fn ctl_arith<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![cpu::ctl_looking_arith_reg(), cpu::ctl_looking_arith_imm()];
    let looked = arith::ctl_looked();
    CrossTableLookup::new(looking, looked)
}

fn ctl_mem<F: Field>() -> CrossTableLookup<F> {
    let looking = (0..N_MEM_CHANNELS).map(cpu::ctl_looking_mem).collect();
    let looked = mem::ctl_looked();
    CrossTableLookup::new(looking, looked)
}
