use core::borrow::Borrow;
use core::cmp::{max, min};
use core::iter::{once, repeat};
use core::marker::PhantomData;

use hashbrown::HashMap;
use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::util::transpose;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::cross_table_lookup::TableWithColumns;
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter, Lookup};
use starky::stark::Stark;

use crate::iter::{windows_mut, LendIter};
use crate::pack::columns::{PackCols, RangeCheck, N_PACK_COLS, PACK_COL_MAP};
use crate::pack::N_BYTES;
use crate::stark::Table;

#[derive(Clone, Debug)]
pub(crate) struct PackOp {
    pub rw: bool,
    pub signed: bool,
    pub adr_virt: u32,
    pub time: u32,
    pub bytes: Vec<u8>,
}

impl PackOp {
    fn into_row<F: Field>(self, map: &mut HashMap<u8, usize>, index: usize) -> PackCols<F> {
        let mut row = PackCols {
            f_rw: F::from_bool(self.rw),
            f_signed: F::from_bool(self.signed),
            adr_virt: F::from_canonical_u32(self.adr_virt),
            time: F::from_canonical_u32(self.time),
            range_check: RangeCheck {
                count: rc_count(index),
                ..Default::default()
            },
            ..Default::default()
        };

        // set index at length of bytes
        let len = self.bytes.len();
        debug_assert!(len > 0 && len <= N_BYTES);
        row.len_idx[len - 1] = F::ONE;

        // self.bytes is big-endian
        let high_byte = self.bytes[0];
        let sign_bit = high_byte >> 7;

        // determine whether extension bits should be set or unset
        let ext_byte = if self.signed && sign_bit != 0 {
            u8::MAX
        } else {
            0
        };
        row.ext_byte = F::from_canonical_u8(ext_byte);

        // deconstruct the most significant byte
        row.high_bits = core::array::from_fn(|i| F::from_bool(high_byte & (1 << i) != 0));

        // write (maybe sign extended) little-endian bytes to row
        row.bytes = self
            .bytes
            .into_iter()
            .rev()
            .chain(repeat(ext_byte))
            .take(N_BYTES)
            .map(|b| {
                let freq = map.entry(b).or_insert(0);
                *freq += 1;
                F::from_canonical_u8(b)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        row
    }
}

pub(crate) fn gen_trace<F: RichField>(
    mut ops: Vec<PackOp>,
    min_rows: usize,
) -> Vec<PolynomialValues<F>> {
    let trace = gen_trace_rows(ops, min_rows);
    let trace_rows: Vec<_> = trace.iter().map(PackCols::to_vec).collect();
    let trace_cols = transpose(&trace_rows);
    trace_cols.into_iter().map(PolynomialValues::new).collect()
}

fn gen_trace_rows<F: RichField>(mut ops: Vec<PackOp>, min_rows: usize) -> Vec<PackCols<F>> {
    let ops_len = ops.iter().filter(|op| !op.bytes.is_empty()).count();
    let n_rows = max(max(ops_len, u8::MAX.into()), min_rows).next_power_of_two();

    // generate rows from nonempty packing ops
    let mut rc_freq = HashMap::default();
    let mut rows: Vec<PackCols<F>> = ops
        .into_iter()
        .filter(|op| !op.bytes.is_empty())
        .enumerate()
        .map(|(i, op)| op.into_row(&mut rc_freq, i))
        .collect();

    // account for padding rows in range check frequencies
    let pad_freq = rc_freq.entry(0).or_insert(0);
    *pad_freq += N_BYTES * n_rows.saturating_sub(ops_len);

    // extend with padding rows
    rows.extend((ops_len..n_rows).map(padding_row));

    // write range check frequencies column
    for (val, freq) in rc_freq {
        rows[val as usize].range_check.freq = F::from_canonical_usize(freq);
    }
    rows
}

fn padding_row<F: Field>(index: usize) -> PackCols<F> {
    PackCols {
        range_check: RangeCheck {
            count: rc_count(index),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn rc_count<F: Field>(index: usize) -> F {
    F::from_canonical_usize(min(index, u8::MAX.into()))
}
