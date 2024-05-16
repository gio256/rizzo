use core::borrow::Borrow;
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter, Lookup};
use starky::stark::Stark;

use crate::mem::columns::{MemCols, MEM_COL_MAP, N_MEM_COLS};

pub(crate) fn ctl_data<F: Field>() -> Vec<Column<F>> {
    Column::singles([
        MEM_COL_MAP.f_rw,
        MEM_COL_MAP.adr_seg,
        MEM_COL_MAP.adr_virt,
        MEM_COL_MAP.val,
        MEM_COL_MAP.time,
    ])
    .collect()
}
pub(crate) fn ctl_filter<F: Field>() -> Filter<F> {
    Filter::new_simple(Column::single(MEM_COL_MAP.f_real))
}

pub(crate) fn eval_all<P: PackedField>(
    lv: &MemCols<P>,
    nv: &MemCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    let flag = lv.f_real;
    cc.constraint(flag * (flag - P::ONES));

    let f_pad = P::ONES - lv.f_real;
    todo!()
}

pub(crate) fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &MemCols<ExtensionTarget<D>>,
    nv: &MemCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}

#[derive(Clone, Copy, Default)]
pub struct MemStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for MemStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_MEM_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_MEM_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_MEM_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &MemCols<P> = local.borrow();
        let next: &[P; N_MEM_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &MemCols<P> = next.borrow();

        eval_all(local, next, cc)
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_MEM_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &MemCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_MEM_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &MemCols<ExtensionTarget<D>> = next.borrow();

        eval_all_circuit(cb, local, next, cc);
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}
