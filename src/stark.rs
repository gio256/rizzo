use core::ops::Deref;

use plonky2::field::extension::Extendable;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use starky::config::StarkConfig;
use starky::cross_table_lookup::{CrossTableLookup, TableIdx, TableWithColumns};
use starky::evaluation_frame::StarkFrame;
use starky::stark::Stark;

use crate::cpu::columns::N_MEM_CHANNELS;
use crate::{alu, cpu, mem};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Table {
    Alu,
    Cpu,
    Mem,
}

fn all_cross_table_lookups<F: Field>() -> Vec<CrossTableLookup<F>> {
    vec![ctl_alu_reg(), ctl_alu_imm(), ctl_mem()]
}

fn ctl_alu_reg<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![cpu::ctl_looking_alu_reg()];
    let looked = alu::ctl_looked_reg();
    CrossTableLookup::new(looking, looked)
}

fn ctl_alu_imm<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![cpu::ctl_looking_alu_imm()];
    let looked = alu::ctl_looked_imm();
    CrossTableLookup::new(looking, looked)
}

fn ctl_mem<F: Field>() -> CrossTableLookup<F> {
    let looking = (0..N_MEM_CHANNELS)
        .map(|ch| {
            TableWithColumns::new(
                Table::Cpu as usize,
                cpu::ctl_looking_mem(ch),
                cpu::ctl_filter_mem(ch),
            )
        })
        .collect();
    let looked = TableWithColumns::new(Table::Mem as usize, mem::ctl_looked(), mem::ctl_filter());
    CrossTableLookup::new(looking, looked)
}
