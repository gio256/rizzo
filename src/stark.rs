use plonky2::field::types::Field;
use starky::cross_table_lookup::CrossTableLookup;

use crate::bytes::BYTES_WORD;
use crate::cpu::columns::N_MEM_CHANNELS;
use crate::{arith, bits, bytes, cpu, mem};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Table {
    Arith,
    Bits,
    Bytes,
    Cpu,
    Mem,
}

fn all_cross_table_lookups<F: Field>() -> Vec<CrossTableLookup<F>> {
    vec![ctl_arith(), ctl_bits(), ctl_bytes(), ctl_mem()]
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

//TODO: Add `bits::stark::ctl_looked_shift` as second looked table.
// See [0xPolygonZero/plonky2/#1575](https://github.com/0xPolygonZero/plonky2/pull/1575)
fn ctl_bits<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![
        cpu::stark::ctl_looking_bits_reg(),
        cpu::stark::ctl_looking_bits_imm(),
    ];
    let looked = bits::stark::ctl_looked_logic();
    CrossTableLookup::new(looking, looked)
}

fn ctl_bytes<F: Field>() -> CrossTableLookup<F> {
    let looking = vec![
        cpu::stark::ctl_looking_read_bytes(),
        cpu::stark::ctl_looking_write_bytes(),
    ];
    let looked = bytes::stark::ctl_looked();
    CrossTableLookup::new(looking, looked)
}

fn ctl_mem<F: Field>() -> CrossTableLookup<F> {
    let cpu = (0..N_MEM_CHANNELS).map(cpu::stark::ctl_looking_mem);
    let bytes = (0..BYTES_WORD).map(bytes::stark::ctl_looking_mem);
    let looking = cpu.chain(bytes).collect();
    let looked = mem::stark::ctl_looked();
    CrossTableLookup::new(looking, looked)
}
