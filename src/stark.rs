use core::ops::Deref;

use plonky2::field::extension::Extendable;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use starky::config::StarkConfig;
use starky::cross_table_lookup::{CrossTableLookup, TableIdx, TableWithColumns};
use starky::evaluation_frame::StarkFrame;
use starky::stark::Stark;

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
