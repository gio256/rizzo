use core::cmp::max;
use core::iter::repeat;

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

use crate::arith::addcy::{self, SIGN_BIT};
use crate::arith::columns::{ArithCols, OpCols, OP_COL_MAP};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Op {
    /// Addition.
    ADD,
    /// Subtraction.
    SUB,
    /// Unsigned less than.
    LTU,
    /// Signed less than.
    LTS,
    /// Unsigned greater than or equal to.
    GEU,
    /// Signed greater than or equal to.
    GES,
}

impl Op {
    fn to_op_cols<F: Field>(self) -> OpCols<F> {
        let mut cols = OpCols::default();
        cols[match self {
            Self::ADD => OP_COL_MAP.f_add,
            Self::SUB => OP_COL_MAP.f_sub,
            Self::LTU => OP_COL_MAP.f_ltu,
            Self::LTS => OP_COL_MAP.f_lts,
            Self::GEU => OP_COL_MAP.f_geu,
            Self::GES => OP_COL_MAP.f_ges,
        }] = F::ONE;
        cols
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ArithOp {
    op: Op,
    in0: u32,
    in1: u32,
}

impl ArithOp {
    pub(crate) fn new(op: Op, in0: u32, in1: u32) -> Self {
        Self { op, in0, in1 }
    }

    fn apply_to_row<F: Field>(self, lv: &mut ArithCols<F>) {
        match self.op {
            Op::ADD => {
                let (res, cy) = self.in0.overflowing_add(self.in1);
                lv.aux = F::from_bool(cy);
                lv.out = F::from_canonical_u32(res);
            }
            Op::SUB => {
                let (diff, cy) = self.in0.overflowing_sub(self.in1);
                lv.aux = F::from_bool(cy);
                lv.out = F::from_canonical_u32(diff);
            }
            Op::LTU => {
                let (diff, lt) = self.in0.overflowing_sub(self.in1);
                lv.aux = F::from_canonical_u32(diff);
                lv.out = F::from_bool(lt);
            }
            Op::GEU => {
                let (diff, lt) = self.in0.overflowing_sub(self.in1);
                lv.aux = F::from_canonical_u32(diff);
                lv.out = F::from_bool(!lt);
            }
            Op::LTS => {
                let (bias0, cy0) = self.in0.overflowing_add(SIGN_BIT);
                let (bias1, cy1) = self.in1.overflowing_add(SIGN_BIT);
                let (diff, lt) = bias0.overflowing_sub(bias1);

                lv.in0_bias = F::from_canonical_u32(bias0);
                lv.in1_bias = F::from_canonical_u32(bias1);
                lv.in0_aux = F::from_bool(cy0);
                lv.in1_aux = F::from_bool(cy1);
                lv.aux = F::from_canonical_u32(diff);
                lv.out = F::from_bool(lt);
            }
            Op::GES => {
                let (bias0, cy0) = self.in0.overflowing_add(SIGN_BIT);
                let (bias1, cy1) = self.in1.overflowing_add(SIGN_BIT);
                let (diff, lt) = bias0.overflowing_sub(bias1);

                lv.in0_bias = F::from_canonical_u32(bias0);
                lv.in1_bias = F::from_canonical_u32(bias1);
                lv.in0_aux = F::from_bool(cy0);
                lv.in1_aux = F::from_bool(cy1);
                lv.aux = F::from_canonical_u32(diff);
                lv.out = F::from_bool(!lt);
            }
        }
    }

    pub(in crate::arith) fn into_row<F: Field>(self) -> ArithCols<F> {
        let mut row = ArithCols {
            op: self.op.to_op_cols(),
            in0: F::from_canonical_u32(self.in0),
            in1: F::from_canonical_u32(self.in1),
            ..Default::default()
        };
        self.apply_to_row(&mut row);
        row
    }
}

pub(crate) fn gen_trace<F: Field>(ops: Vec<ArithOp>, min_rows: usize) -> Vec<PolynomialValues<F>> {
    let trace = gen_trace_rows(ops, min_rows);
    let trace_rows: Vec<_> = trace.map(|col| col.to_vec()).collect();
    let trace_cols = transpose(&trace_rows);
    trace_cols.into_iter().map(PolynomialValues::new).collect()
}

fn gen_trace_rows<F: Field>(
    ops: Vec<ArithOp>,
    min_rows: usize,
) -> impl Iterator<Item = ArithCols<F>> {
    let n_rows = max(ops.len(), min_rows).next_power_of_two();
    ops.into_iter()
        .map(ArithOp::into_row)
        .chain(repeat(ArithCols::default()))
        .take(n_rows)
}
