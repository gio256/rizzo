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

use crate::logic::columns::{LogicCols, LOGIC_COL_MAP, N_LOGIC_COLS};
use crate::stark::Table;
use crate::util::fst;
use crate::vm::opcode::Opcode;

/// Operation flags and the corresponding opcode.
const LOGIC_OPS: [(usize, u8); 3] = [
    (LOGIC_COL_MAP.op.f_and, Opcode::AND as u8),
    (LOGIC_COL_MAP.op.f_xor, Opcode::XOR as u8),
    (LOGIC_COL_MAP.op.f_or, Opcode::OR as u8),
];

pub(crate) fn ctl_looked<F: Field>() -> TableWithColumns<F> {
    let op_comb = LOGIC_OPS.map(|(f, op)| (f, F::from_canonical_u8(op)));
    let op = Column::linear_combination(op_comb);
    let in0 = Column::le_bits(LOGIC_COL_MAP.in0);
    let in1 = Column::le_bits(LOGIC_COL_MAP.in1);
    let out = Column::single(LOGIC_COL_MAP.out);

    let cols = vec![op, in0, in1, out];
    let filter = Filter::new_simple(Column::sum(LOGIC_OPS.map(fst)));
    TableWithColumns::new(Table::Logic as usize, cols, filter)
}

fn eval_all<P: PackedField>(lv: &LogicCols<P>, nv: &LogicCols<P>, cc: &mut ConstraintConsumer<P>) {
    let f_and = lv.op.f_and;
    let f_xor = lv.op.f_xor;
    let f_or = lv.op.f_or;

    // flags in {0, 1}
    cc.constraint(f_and * (f_and - P::ONES));
    cc.constraint(f_xor * (f_xor - P::ONES));
    cc.constraint(f_or * (f_or - P::ONES));

    // at most one op flag is set
    let flag_sum: P = f_and + f_xor + f_or;
    cc.constraint(flag_sum * (flag_sum - P::ONES));

    // input bit values in {0, 1}
    for bit in lv.in0 {
        cc.constraint(bit * (bit - P::ONES));
    }
    for bit in lv.in1 {
        cc.constraint(bit * (bit - P::ONES));
    }

    //TODO
}

fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &LogicCols<ExtensionTarget<D>>,
    nv: &LogicCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LogicStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for LogicStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_LOGIC_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget =
        StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_LOGIC_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_LOGIC_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &LogicCols<P> = local.borrow();
        let next: &[P; N_LOGIC_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &LogicCols<P> = next.borrow();
        eval_all(local, next, cc);
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_LOGIC_COLS] =
            frame.get_local_values().try_into().unwrap();
        let local: &LogicCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_LOGIC_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &LogicCols<ExtensionTarget<D>> = next.borrow();
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

    use super::LogicStark;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type S = LogicStark<F, D>;

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
