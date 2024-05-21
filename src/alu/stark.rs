use core::borrow::Borrow;
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::cross_table_lookup::TableWithColumns;
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter};
use starky::stark::Stark;

use crate::alu::addcy;
use crate::alu::columns::{AluCols, ALU_COL_MAP, N_ALU_COLS};
use crate::stark::Table;
use crate::vm::opcode::Opcode;

const ALU_OPS: [(usize, u8); 3] = [
    (ALU_COL_MAP.op.f_add, Opcode::ADD as u8),
    (ALU_COL_MAP.op.f_sub, Opcode::SUB as u8),
    (ALU_COL_MAP.op.f_lt, Opcode::SLT as u8),
];

pub(crate) fn ctl_looked<F: Field>() -> TableWithColumns<F> {
    // the first column evaluates to the opcode of the selected instruction
    let ops = ALU_OPS.iter().map(|&(f, op)| (f, F::from_canonical_u8(op)));
    let mut cols = vec![Column::linear_combination(ops)];
    cols.extend(Column::singles([
        ALU_COL_MAP.in0,
        ALU_COL_MAP.in1,
        ALU_COL_MAP.out,
    ]));

    let f_alu = Column::sum(ALU_OPS.iter().map(|&(f, _)| f));
    let filter = Filter::new_simple(f_alu);
    TableWithColumns::new(Table::Alu as usize, cols, filter)
}

#[derive(Clone, Copy, Default)]
pub(crate) struct AluStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

fn eval_all<P: PackedField>(lv: &AluCols<P>, nv: &AluCols<P>, cc: &mut ConstraintConsumer<P>) {
    // cc.constraint(f_add * (f_add - P::ONES));
    // cc.constraint(f_sub * (f_sub - P::ONES));
    // cc.constraint(f_lt * (f_lt - P::ONES));
    addcy::eval(lv, cc)
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

    fn requires_ctls(&self) -> bool {
        true
    }
}
