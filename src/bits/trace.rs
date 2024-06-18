use core::cmp::max;
use core::iter::repeat;

use hashbrown::HashMap;
use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::field::types::Field;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::util::transpose;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::cross_table_lookup::TableWithColumns;
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter, Lookup};
use starky::stark::Stark;

use crate::bits::columns::{BitCols, OpCols, N_BIT_COLS, OP_COL_MAP, WORD_BITS};
use crate::util::u32_to_le_bits;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Op {
    AND,
    OR,
    XOR,
    SLL,
    SRL,
    SRA,
}

impl Op {
    fn apply(self, x: u32, y: u32) -> u32 {
        match self {
            Self::AND => x & y,
            Self::OR => x | y,
            Self::XOR => x ^ y,
            Self::SLL => x << y,
            Self::SRL => x >> y,
            Self::SRA => (x as i32 >> y) as u32,
        }
    }

    fn to_op_cols<F: Field>(self) -> OpCols<F> {
        let mut cols = OpCols::default();
        cols[match self {
            Self::AND => OP_COL_MAP.f_and,
            Self::OR => OP_COL_MAP.f_or,
            Self::XOR => OP_COL_MAP.f_xor,
            Self::SLL => OP_COL_MAP.f_sll,
            Self::SRL => OP_COL_MAP.f_srl,
            Self::SRA => OP_COL_MAP.f_sra,
        }] = F::ONE;
        cols
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BitOp {
    op: Op,
    in0: u32,
    in1: u32,
}

impl BitOp {
    pub(crate) fn new(op: Op, in0: u32, in1: u32) -> Self {
        Self { op, in0, in1 }
    }

    fn into_row<F: Field>(self) -> BitCols<F> {
        let in1 = match self.op {
            Op::AND | Op::OR | Op::XOR => u32_to_le_bits(self.in1),
            Op::SLL | Op::SRL | Op::SRA => {
                assert!(self.in1 < WORD_BITS as u32);
                let mut res = [F::ZERO; WORD_BITS];
                res[self.in1 as usize] = F::ONE;
                res
            }
        };
        BitCols {
            op: self.op.to_op_cols(),
            in0: u32_to_le_bits(self.in0),
            out: F::from_canonical_u32(self.op.apply(self.in0, self.in1)),
            and: F::from_canonical_u32(self.in0 & self.in1),
            in1,
        }
    }
}

pub(crate) fn gen_trace<F: Field>(ops: Vec<BitOp>, min_rows: usize) -> Vec<PolynomialValues<F>> {
    let trace = gen_trace_rows(ops, min_rows);
    let trace_rows: Vec<_> = trace.map(|col| col.to_vec()).collect();
    let trace_cols = transpose(&trace_rows);
    trace_cols.into_iter().map(PolynomialValues::new).collect()
}

fn gen_trace_rows<F: Field>(ops: Vec<BitOp>, min_rows: usize) -> impl Iterator<Item = BitCols<F>> {
    let n_rows = max(ops.len(), min_rows).next_power_of_two();
    ops.into_iter()
        .map(BitOp::into_row)
        .chain(repeat(BitCols::default()))
        .take(n_rows)
}
