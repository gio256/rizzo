use core::ops::Deref;

use plonky2::field::extension::Extendable;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use starky::config::StarkConfig;
use starky::cross_table_lookup::{CrossTableLookup, TableIdx, TableWithColumns};
use starky::evaluation_frame::StarkFrame;
use starky::stark::Stark;

use crate::cpu::columns::N_MEM_CHANNELS;
use crate::{cpu, mem};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Table {
    Alu = 0,
    Cpu = 1,
    Mem = 2,
}
impl Deref for Table {
    type Target = TableIdx;
    fn deref(&self) -> &Self::Target {
        [&0, &1, &2][*self as TableIdx]
    }
}

fn all_cross_table_lookups<F: Field>() -> Vec<CrossTableLookup<F>> {
    vec![ctl_mem()]
}

fn ctl_mem<F: Field>() -> CrossTableLookup<F> {
    let cpu = (0..N_MEM_CHANNELS)
        .map(|ch| {
            TableWithColumns::new(
                *Table::Cpu,
                cpu::ctl_looking_mem(ch),
                cpu::ctl_filter_mem(ch),
            )
        })
        .collect();
    let looked = TableWithColumns::new(*Table::Mem, mem::ctl_looked(), mem::ctl_filter());
    CrossTableLookup::new(cpu, looked)
}
