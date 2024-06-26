//! Constraints for ADD, SUB, SLT, SLTU, and branching comparisons.
//!
//! This is essentially [zk_evm]'s "add with carry out" implementation, except
//! that we only have one limb to deal with.
//!
//! [zk_evm]: https://github.com/0xPolygonZero/zk_evm/blob/develop/evm_arithmetization/src/arithmetic/addcy.rs

use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::arith::columns::ArithCols;

/// The multiplicative inverse of 2^32.
const GOLDILOCKS_INVERSE_REG_SIZE: u64 = 18446744065119617026;
const REG_BITS: usize = 32;
pub(in crate::arith) const SIGN_BIT: u32 = 1 << (REG_BITS - 1);

/// See [zkevm] for more on the signed comparison method used here.
///
/// [zkevm]: https://github.com/0xPolygonZero/zk_evm/blob/e8e60717efd5eadc6d84d8c59902f40806d7c770/evm_arithmetization/src/cpu/kernel/asm/signed.asm#L156-L161
pub(crate) fn eval<P: PackedField>(lv: &ArithCols<P>, cc: &mut ConstraintConsumer<P>) {
    let in0 = lv.in0;
    let in1 = lv.in1;
    let out = lv.out;
    let aux = lv.aux;

    // Eval addition.
    eval_add(cc, lv.op.f_add, in0, in1, out, aux);

    // Eval subtraction.
    eval_sub(cc, lv.op.f_sub, in0, in1, out, aux);

    // Eval unsigned less than.
    eval_ltu(cc, lv.op.f_ltu, in0, in1, out, aux);

    // Eval unsigned greater than or equal to.
    let not_out = P::ONES - out;
    eval_ltu(cc, lv.op.f_geu, in0, in1, not_out, aux);

    // Eval signed less than and signed greater than or equal to.
    //
    // x <s y iff (x ^ sign_bit) <u (y ^ sign_bit)
    //
    // where <s is signed less than and <u is unsigned less than.
    // Note that we can also replace xor in the above equation with addition.
    // Reference: Hacker's Delight, 2nd edition, §2-12, via zk_evm
    let f_lts = lv.op.f_lts;
    let f_ges = lv.op.f_ges;
    let f_signed = f_lts + f_ges;
    let in0_bias = lv.in0_bias;
    let in1_bias = lv.in1_bias;
    let sign_bit: P = P::Scalar::from_canonical_u32(SIGN_BIT).into();

    // in0 + 2^31 == in0_bias
    eval_add(cc, f_signed, in0, sign_bit, in0_bias, lv.in0_aux);

    // in1 + 2^31 == in1_bias
    eval_add(cc, f_signed, in1, sign_bit, in1_bias, lv.in1_aux);

    // in0_bias <u in1_bias == out
    eval_ltu(cc, f_lts, in0_bias, in1_bias, out, aux);

    // in0_bias <u in1_bias == 1 - out
    eval_ltu(cc, f_ges, in0_bias, in1_bias, not_out, aux);
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &ArithCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
}

/// Constrains `x + y == z + cy*2^32` if `filter != 0`.
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

    // did_cy matches cy
    if transition {
        cc.constraint_transition(filter * (did_cy - cy));
    } else {
        cc.constraint(filter * (did_cy - cy));
    }
}

/// Constrains `x + y == z + cy*2^32` if `filter != 0`.
#[allow(clippy::too_many_arguments)]
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

/// `left + right == out`
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

/// `left + right == out`
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

/// `left - right == out`
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

/// `left <u right == out` (unsigned).
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

#[cfg(test)]
mod tests {
    use core::borrow::BorrowMut;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::field::types::Sample;
    use rand::Rng;

    use crate::arith::columns::N_ARITH_COLS;
    use crate::arith::trace::{ArithOp, Op};

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
    fn test_eval_not_addcy() {
        let mut rng = rand::thread_rng();
        let mut lv = [F::default(); N_ARITH_COLS].map(|_| F::sample(&mut rng));
        let lv: &mut ArithCols<F> = lv.borrow_mut();

        // turn all operation flags off
        lv.op.iter_mut().for_each(|f| *f = F::ZERO);

        let mut cc = constraint_consumer();
        eval(lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }
    }

    #[test]
    fn test_gen_eval_add() {
        let mut rng = rand::thread_rng();
        let left = rng.gen();
        let right = rng.gen();
        let op = ArithOp::new(Op::ADD, left, right);
        let lv = op.into_row();

        let mut cc = constraint_consumer();
        eval(&lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let (expect, cy) = left.overflowing_add(right);
        assert_eq!(lv.out, F::from_canonical_u32(expect));
        assert_eq!(lv.aux, F::from_canonical_u32(cy as u32));
    }

    #[test]
    fn test_gen_eval_sub() {
        let mut rng = rand::thread_rng();
        let left: u32 = rng.gen();
        let right: u32 = rng.gen();
        let op = ArithOp::new(Op::SUB, left, right);
        let lv = op.into_row();

        let mut cc = constraint_consumer();
        eval(&lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let (expect, cy) = left.overflowing_sub(right);
        assert_eq!(lv.out, F::from_canonical_u32(expect));
        assert_eq!(lv.aux, F::from_canonical_u32(cy as u32));
    }

    #[test]
    fn test_gen_eval_ltu() {
        let mut rng = rand::thread_rng();
        let left: u32 = rng.gen();
        let right: u32 = rng.gen();
        let op = ArithOp::new(Op::LTU, left, right);
        let lv = op.into_row();

        let mut cc = constraint_consumer();
        eval(&lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let expect = left < right;
        assert_eq!(lv.out, F::from_bool(expect));
    }

    #[test]
    fn test_gen_eval_geu() {
        let mut rng = rand::thread_rng();
        let left: u32 = rng.gen();
        let right: u32 = rng.gen();
        let op = ArithOp::new(Op::GEU, left, right);
        let lv = op.into_row();

        let mut cc = constraint_consumer();
        eval(&lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let expect = left >= right;
        assert_eq!(lv.out, F::from_bool(expect));
    }

    #[test]
    fn test_gen_eval_lts() {
        let mut rng = rand::thread_rng();
        let left: i32 = rng.gen();
        let right: i32 = rng.gen();
        let op = ArithOp::new(Op::LTS, left as u32, right as u32);
        let lv = op.into_row();

        let mut cc = constraint_consumer();
        eval(&lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let expect = left < right;
        assert_eq!(lv.out, F::from_bool(expect));
    }

    #[test]
    fn test_gen_eval_ges() {
        let mut rng = rand::thread_rng();
        let left: i32 = rng.gen();
        let right: i32 = rng.gen();
        let op = ArithOp::new(Op::GES, left as u32, right as u32);
        let lv = op.into_row();

        let mut cc = constraint_consumer();
        eval(&lv, &mut cc);
        for acc in cc.accumulators() {
            assert_eq!(acc, F::ZERO);
        }

        let expect = left >= right;
        assert_eq!(lv.out, F::from_bool(expect));
    }
}
