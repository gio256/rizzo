use core::cmp::{max, min};
use core::iter::repeat;

use hashbrown::HashMap;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::field::types::Field;
use plonky2::util::transpose;

use crate::bytes::columns::{ByteCols, RangeCheck};
use crate::bytes::BYTES_WORD;

#[derive(Clone, Debug)]
pub(crate) struct ByteOp {
    pub rw: bool,
    pub signed: bool,
    pub adr_virt: u32,
    pub time: u32,
    pub bytes: Vec<u8>,
}

impl ByteOp {
    fn into_row<F: Field>(self, map: &mut HashMap<u8, usize>, index: usize) -> ByteCols<F> {
        let mut row = ByteCols {
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
        debug_assert!(len > 0 && len <= BYTES_WORD);
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
        row.high_bits = crate::util::u8_to_le_bits(high_byte);

        // write (maybe sign extended) little-endian bytes to row
        row.bytes = self
            .bytes
            .into_iter()
            .rev()
            .chain(repeat(ext_byte))
            .take(BYTES_WORD)
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

pub(crate) fn gen_trace<F: Field>(ops: Vec<ByteOp>, min_rows: usize) -> Vec<PolynomialValues<F>> {
    let trace = gen_trace_rows(ops, min_rows);
    let trace_rows: Vec<_> = trace.iter().map(ByteCols::to_vec).collect();
    let trace_cols = transpose(&trace_rows);
    trace_cols.into_iter().map(PolynomialValues::new).collect()
}

fn gen_trace_rows<F: Field>(ops: Vec<ByteOp>, min_rows: usize) -> Vec<ByteCols<F>> {
    let n_ops = ops.iter().filter(|op| !op.bytes.is_empty()).count();
    let n_rows = max(max(n_ops, u8::MAX.into()), min_rows).next_power_of_two();

    // generate rows from nonempty byte packing ops
    let mut rc_freq = HashMap::default();
    let mut rows: Vec<ByteCols<F>> = ops
        .into_iter()
        .filter(|op| !op.bytes.is_empty())
        .enumerate()
        .map(|(i, op)| op.into_row(&mut rc_freq, i))
        .chain((n_ops..n_rows).map(padding_row))
        .collect();

    // account for padding rows in range check frequencies
    let pad_freq = rc_freq.entry(0).or_insert(0);
    *pad_freq += BYTES_WORD * n_rows.saturating_sub(n_ops);

    // write range check frequencies column
    for (val, freq) in rc_freq {
        rows[val as usize].range_check.freq = F::from_canonical_usize(freq);
    }
    rows
}

fn padding_row<F: Field>(index: usize) -> ByteCols<F> {
    ByteCols {
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
