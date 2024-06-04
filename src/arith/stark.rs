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

use crate::arith::addcy;
use crate::arith::columns::{ArithCols, ARITH_COL_MAP, N_ARITH_COLS};
use crate::stark::Table;
use crate::util::fst;
use crate::vm::opcode::Opcode;

const ARITH_OPS: [(usize, u8); 3] = [
    (ARITH_COL_MAP.op.f_add, Opcode::Add as u8),
    (ARITH_COL_MAP.op.f_sub, Opcode::Sub as u8),
    (ARITH_COL_MAP.op.f_ltu, Opcode::Sltu as u8),
];

pub(crate) fn ctl_looked<F: Field>() -> TableWithColumns<F> {
    // the first column evaluates to the opcode of the selected instruction
    let op_comb = ARITH_OPS.map(|(f, op)| (f, F::from_canonical_u8(op)));
    let mut cols = vec![Column::linear_combination(op_comb)];
    cols.extend(Column::singles([
        ARITH_COL_MAP.in0,
        ARITH_COL_MAP.in1,
        ARITH_COL_MAP.out,
    ]));

    let filter = Filter::new_simple(Column::sum(ARITH_OPS.map(fst)));
    TableWithColumns::new(Table::Arith as usize, cols, filter)
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ArithStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

fn eval_all<P: PackedField>(lv: &ArithCols<P>, nv: &ArithCols<P>, cc: &mut ConstraintConsumer<P>) {
    addcy::eval(lv, cc)
}

fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &ArithCols<ExtensionTarget<D>>,
    nv: &ArithCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    addcy::eval_circuit(cb, lv, nv, cc);
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for ArithStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_ARITH_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget =
        StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_ARITH_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_ARITH_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &ArithCols<P> = local.borrow();
        let next: &[P; N_ARITH_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &ArithCols<P> = next.borrow();
        eval_all(local, next, cc);
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_ARITH_COLS] =
            frame.get_local_values().try_into().unwrap();
        let local: &ArithCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_ARITH_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &ArithCols<ExtensionTarget<D>> = next.borrow();
        eval_all_circuit(cb, local, next, cc);
    }

    fn constraint_degree(&self) -> usize {
        3
    }

    fn requires_ctls(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use starky::stark_testing::{test_stark_circuit_constraints, test_stark_low_degree};

    use super::ArithStark;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type S = ArithStark<F, D>;

    #[test]
    fn test_stark_degree() {
        let stark: S = Default::default();
        test_stark_low_degree(stark).unwrap();
    }

    // #[test]
    // fn test_stark_circuit() {
    //     let stark: S = Default::default();
    //     test_stark_circuit_constraints::<F, C, S, D>(stark).unwrap();
    // }
}
