use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::arith::columns::{ArithCols, ARITH_COL_MAP};

/// The multiplicative inverse of 2^32.
const GOLDILOCKS_INVERSE_REG_SIZE: u64 = 18446744065119617026;
const REG_BITS: usize = 32;
// const SIGN_BIT: u32 = 1 << (REG_BITS - 1);

pub(crate) fn generate<F: PrimeField64>(
    lv: &mut ArithCols<F>,
    filter: usize,
    left: u32,
    right: u32,
) {
    lv.in0 = F::from_canonical_u32(left);
    lv.in1 = F::from_canonical_u32(right);

    if filter == ARITH_COL_MAP.op.f_add {
        let (res, cy) = left.overflowing_add(right);
        lv.aux = F::from_canonical_u32(cy as u32);
        lv.out = F::from_canonical_u32(res);
    } else if filter == ARITH_COL_MAP.op.f_sub {
        let (diff, cy) = left.overflowing_sub(right);
        lv.aux = F::from_canonical_u32(cy as u32);
        lv.out = F::from_canonical_u32(diff);
    } else if filter == ARITH_COL_MAP.op.f_ltu {
        let (diff, cy) = left.overflowing_sub(right);
        lv.aux = F::from_canonical_u32(diff);
        lv.out = F::from_canonical_u32(cy as u32);
    } else {
        panic!("bad instruction filter")
    };
}

pub(crate) fn eval<P: PackedField>(lv: &ArithCols<P>, cc: &mut ConstraintConsumer<P>) {
    let f_add = lv.op.f_add;
    let f_sub = lv.op.f_sub;
    let f_ltu = lv.op.f_ltu;

    let in0 = lv.in0;
    let in1 = lv.in1;
    let out = lv.out;
    let aux = lv.aux;

    eval_add(cc, f_add, in0, in1, out, aux);
    eval_sub(cc, f_sub, in0, in1, out, aux);
    eval_ltu(cc, f_ltu, in0, in1, out, aux);
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &ArithCols<ExtensionTarget<D>>,
    nv: &ArithCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
    let f_add = lv.op.f_add;
    let f_sub = lv.op.f_sub;
    let f_ltu = lv.op.f_ltu;

    let in0 = lv.in0;
    let in1 = lv.in1;
    let out = lv.out;
    let aux = lv.aux;
}

/// Constrains x + y == z + cy * 2^32
fn eval_addcy<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    x: P,
    y: P,
    z: P,
    cy: P,
    transition: bool,
) {
    let base = P::Scalar::from_canonical_u64(1u64 << REG_BITS);
    let base_inv = P::Scalar::from_canonical_u64(GOLDILOCKS_INVERSE_REG_SIZE);
    debug_assert!(base * base_inv == P::Scalar::ONE);

    // diff in {0, base}
    let diff = x + y - z;
    if transition {
        cc.constraint_transition(filter * diff * (diff - base));
    } else {
        cc.constraint(filter * diff * (diff - base));
    }

    // did_cy in {0, 1}
    let did_cy = diff * base_inv;
    cc.constraint(filter * cy * (cy - P::ONES));
    if transition {
        cc.constraint_transition(filter * (did_cy - cy));
    } else {
        cc.constraint(filter * (did_cy - cy));
    }
}

fn eval_addcy_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
    filter: ExtensionTarget<D>,
    x: ExtensionTarget<D>,
    y: ExtensionTarget<D>,
    z: ExtensionTarget<D>,
    cy: ExtensionTarget<D>,
    transition: bool,
) {
    let base = F::from_canonical_u64(1u64 << REG_BITS);
    let base_inv = F::from_canonical_u64(GOLDILOCKS_INVERSE_REG_SIZE);
    //TODO
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
    eval_addcy(cc, filter, left, right, out, overflow, false)
}

pub(crate) fn eval_add_transition<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    left: P,
    right: P,
    out: P,
    overflow: P,
) {
    // constrain left + right == out + overflow * 2^32
    eval_addcy(cc, filter, left, right, out, overflow, true)
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
    eval_addcy(cc, filter, right, out, left, overflow, false)
}

pub(crate) fn eval_ltu<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    left: P,
    right: P,
    out: P,
    diff: P,
) {
    // constrain right + diff == left + out * 2^32
    eval_addcy(cc, filter, right, diff, left, out, false)
}

pub(crate) fn eval_gtu<P: PackedField>(
    cc: &mut ConstraintConsumer<P>,
    filter: P,
    left: P,
    right: P,
    out: P,
    diff: P,
) {
    // constrain right + diff == left + out * 2^32
    eval_addcy(cc, filter, left, diff, right, out, false)
}

// pub(crate) fn eval_lts<P: PackedField>(
//     cc: &mut ConstraintConsumer<P>,
//     filter: P,
//     left: P,
//     right: P,
//     out: P,
//     left_bias: P,
//     left_bias_ovf: P,
//     right_bias: P,
//     right_bias_ovf: P,
//     diff: P,
// ) {
//     let sign_mask: P = P::Scalar::from_canonical_u32(SIGN_BIT).into();
//     eval_add(cc, filter, left, sign_mask, left_bias, left_bias_ovf);
//     eval_add(cc, filter, right, sign_mask, right_bias, right_bias_ovf);
//     eval_ltu(cc, filter, left_bias, right_bias, out, diff);
// }

#[cfg(test)]
mod tests {
    use core::borrow::BorrowMut;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::field::types::Sample;
    use rand::{Rng, SeedableRng};

    use crate::arith::columns::N_ARITH_COLS;

    use super::*;

    type F = GoldilocksField;

    fn constraint_consumer() -> ConstraintConsumer<F> {
        ConstraintConsumer::new(
            vec![GoldilocksField(2), GoldilocksField(3), GoldilocksField(5)],
            F::ONE,
            F::ONE,
            F::ONE,
        )
    }

    #[test]
    fn eval_not_addcy() {
        let mut rng = rand::thread_rng();
        let mut lv = [F::default(); N_ARITH_COLS].map(|_| F::sample(&mut rng));
        let lv: &mut ArithCols<F> = lv.borrow_mut();

        lv.op.f_add = F::ZERO;
        lv.op.f_sub = F::ZERO;
        lv.op.f_ltu = F::ZERO;

        let mut cc = constraint_consumer();
        eval(lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }
    }

    #[test]
    fn generate_eval_add() {
        let mut rng = rand::thread_rng();
        let mut lv = [F::default(); N_ARITH_COLS].map(|_| F::sample(&mut rng));
        let lv: &mut ArithCols<F> = lv.borrow_mut();

        lv.op.f_add = F::ONE;
        lv.op.f_sub = F::ZERO;
        lv.op.f_ltu = F::ZERO;

        let left: u32 = rng.gen();
        let right: u32 = rng.gen();
        generate(lv, ARITH_COL_MAP.op.f_add, left, right);

        let mut cc = constraint_consumer();
        eval(lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let (expect, cy) = left.overflowing_add(right);
        assert_eq!(lv.out, F::from_canonical_u32(expect));
        assert_eq!(lv.aux, F::from_canonical_u32(cy as u32));
    }

    #[test]
    fn generate_eval_sub() {
        let mut rng = rand::thread_rng();
        let mut lv = [F::default(); N_ARITH_COLS].map(|_| F::sample(&mut rng));
        let lv: &mut ArithCols<F> = lv.borrow_mut();

        lv.op.f_add = F::ZERO;
        lv.op.f_sub = F::ONE;
        lv.op.f_ltu = F::ZERO;

        let left: u32 = rng.gen();
        let right: u32 = rng.gen();
        generate(lv, ARITH_COL_MAP.op.f_sub, left, right);

        let mut cc = constraint_consumer();
        eval(lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let (expect, cy) = left.overflowing_sub(right);
        assert_eq!(lv.out, F::from_canonical_u32(expect));
        assert_eq!(lv.aux, F::from_canonical_u32(cy as u32));
    }

    #[test]
    fn generate_eval_ltu() {
        let mut rng = rand::thread_rng();
        let mut lv = [F::default(); N_ARITH_COLS].map(|_| F::sample(&mut rng));
        let lv: &mut ArithCols<F> = lv.borrow_mut();

        lv.op.f_add = F::ZERO;
        lv.op.f_sub = F::ZERO;
        lv.op.f_ltu = F::ONE;

        let left: u32 = rng.gen();
        let right: u32 = rng.gen();
        generate(lv, ARITH_COL_MAP.op.f_ltu, left, right);

        let mut cc = constraint_consumer();
        eval(lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let expect = left < right;
        assert_eq!(lv.out, F::from_canonical_u32(expect as u32));
    }
}