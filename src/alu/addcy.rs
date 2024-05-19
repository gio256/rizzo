use core::borrow::Borrow;
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::stark::Stark;

/// The multiplicative inverse of 2^32.
const GOLDILOCKS_INVERSE_REG_SIZE: u64 = 18446744065119617026;
const REG_BITS: usize = 32;

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
