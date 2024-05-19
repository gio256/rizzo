use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::alu::columns::{AluCols, ALU_COL_MAP};

/// The multiplicative inverse of 2^32.
const GOLDILOCKS_INVERSE_REG_SIZE: u64 = 18446744065119617026;
const REG_BITS: usize = 32;

pub(crate) fn generate<F: PrimeField64>(lv: &mut AluCols<F>, filter: usize, left: u32, right: u32) {
    lv.in0 = F::from_canonical_u32(left);
    lv.in1 = F::from_canonical_u32(right);

    let f_add = ALU_COL_MAP.f_add;
    let f_sub = ALU_COL_MAP.f_sub;
    let f_lt = ALU_COL_MAP.f_lt;

    match filter {
        f_add => {
            let (res, cy) = left.overflowing_add(right);
            lv.aux = F::from_canonical_u32(cy as u32);
            lv.out = F::from_canonical_u32(res);
        }
        f_sub => {
            let (diff, cy) = left.overflowing_sub(right);
            lv.aux = F::from_canonical_u32(cy as u32);
            lv.out = F::from_canonical_u32(diff);
        }
        f_lt => {
            let (diff, cy) = left.overflowing_sub(right);
            lv.aux = F::from_canonical_u32(diff);
            lv.out = F::from_canonical_u32(cy as u32);
        }
        _ => panic!("bad instruction filter")
    };
}

/// Constrains x + y == z + cy * 2^32
fn eval_addcy<P: PackedField>(cc: &mut ConstraintConsumer<P>, filter: P, x: P, y: P, z: P, cy: P) {
    let base = P::Scalar::from_canonical_u64(1u64 << REG_BITS);
    let base_inv = P::Scalar::from_canonical_u64(GOLDILOCKS_INVERSE_REG_SIZE);
    debug_assert!(base * base_inv == P::Scalar::ONE);

    // diff in {0, base}
    let diff = x + y - z;
    cc.constraint(filter * diff * (diff - base));

    // did_cy in {0, 1}
    let did_cy = diff * base_inv;
    cc.constraint(filter * cy * (cy - P::ONES));
    cc.constraint(filter * (did_cy - cy));
}

fn eval_addcy_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
    filter: ExtensionTarget<D>,
    x: ExtensionTarget<D>,
    y: ExtensionTarget<D>,
    z: ExtensionTarget<D>,
    cy: ExtensionTarget<D>,
) {
    todo!()
}

pub(crate) fn eval_add<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    left: P,
    right: P,
    out: P,
    overflow: P,
) {
    // constrain left + right == out + overflow * 2^32
    eval_addcy(cc, filter, left, right, out, overflow)
}

pub(crate) fn eval_sub<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    left: P,
    right: P,
    out: P,
    overflow: P,
) {
    // constrain right + out == left + overflow * 2^32
    eval_addcy(cc, filter, right, out, left, overflow)
}

pub(crate) fn eval_lt<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    left: P,
    right: P,
    out: P,
    diff: P,
) {
    // constrain right + diff == left + out * 2^32
    eval_addcy(cc, filter, right, diff, left, out)
}

pub(crate) fn eval_gt<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    left: P,
    right: P,
    out: P,
    diff: P,
) {
    // constrain right + diff == left + out * 2^32
    eval_addcy(cc, filter, left, diff, right, out)
}
