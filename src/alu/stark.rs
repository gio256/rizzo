use core::borrow::Borrow;
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
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

fn eval_all<P: PackedField>(lv: &AluCols<P>, nv: &AluCols<P>, cc: &mut ConstraintConsumer<P>) {
    let is_add = lv.is_add;
    let is_sub = lv.is_sub;
    cc.constraint(is_add * (is_add - P::ONES));
    cc.constraint(is_sub * (is_sub - P::ONES));
    cc.constraint(is_add * (lv.out - lv.in0 - lv.in1));
    cc.constraint(is_sub * (lv.out - lv.in0 + lv.in1));
}

pub(crate) fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
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
