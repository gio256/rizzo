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

use crate::alu::columns::{AluCols, N_ALU_COLS};

#[derive(Clone, Copy, Default)]
pub struct AluStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

/// The multiplicative inverse of (1 << 32).
const GOLDILOCKS_INVERSE_U32_MAX: u64 = 18446744065119617026;

/// Constrains x + y == z + cy * 2^32
fn eval_addcy<P: PackedField>(cc: &mut ConstraintConsumer<P>, filter: P, x: P, y: P, z: P, cy: P) {
    let base = P::Scalar::from_canonical_u64(1u64 << 32);
    let base_inv = P::Scalar::from_canonical_u64(GOLDILOCKS_INVERSE_U32_MAX);
    debug_assert!(base * base_inv == P::Scalar::ONE);

    // diff in {0, base}
    let diff = x + y - z;
    cc.constraint(filter * diff * (diff - base));

    // did_cy in {0, 1}
    let did_cy = diff * base_inv;
    cc.constraint(filter * cy * (cy - P::ONES));
    cc.constraint(filter * (did_cy - cy));
}

fn eval_all<P: PackedField>(lv: &AluCols<P>, nv: &AluCols<P>, cc: &mut ConstraintConsumer<P>) {
    let f_add = lv.f_add;
    let f_sub = lv.f_sub;
    let f_lt = lv.f_lt;
    cc.constraint(f_add * (f_add - P::ONES));
    cc.constraint(f_sub * (f_sub - P::ONES));
    cc.constraint(f_lt * (f_lt - P::ONES));

    let in0 = lv.in0;
    let in1 = lv.in1;
    let out = lv.out;
    let aux = lv.aux;

    // constrain in0 + in1 == out + aux * 2^32
    eval_addcy(cc, f_add, in0, in1, out, aux);
    // constrain in1 + out == in0 + aux * 2^32
    eval_addcy(cc, f_sub, in1, out, in0, aux);
    // constrain in1 + aux == in0 + out * 2^32
    eval_addcy(cc, f_lt, in1, aux, in0, out);
}

fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &AluCols<ExtensionTarget<D>>,
    nv: &AluCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for AluStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_ALU_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_ALU_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_ALU_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &AluCols<P> = local.borrow();
        let next: &[P; N_ALU_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &AluCols<P> = next.borrow();

        eval_all(local, next, cc);
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_ALU_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &AluCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_ALU_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &AluCols<ExtensionTarget<D>> = next.borrow();

        eval_all_circuit(cb, local, next, cc);
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}
