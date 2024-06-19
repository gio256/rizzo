use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::arith::columns::ArithCols;

pub(crate) fn eval<P: PackedField>(lv: &ArithCols<P>, cc: &mut ConstraintConsumer<P>) {
    for flag in *lv.op {
        cc.constraint(flag * (flag - P::ONES));
    }
    let flag_sum: P = lv.op.into_iter().sum();
    cc.constraint(flag_sum * (flag_sum - P::ONES));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &ArithCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
}
