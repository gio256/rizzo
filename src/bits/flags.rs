use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::bits::columns::BitCols;

pub(crate) fn eval<P: PackedField>(lv: &BitCols<P>, cc: &mut ConstraintConsumer<P>) {
    // flags in {0, 1}
    for flag in *lv.op {
        cc.constraint(flag * (flag - P::ONES));
    }

    // at most one op flag is set
    let flag_sum: P = lv.op.into_iter().sum();
    cc.constraint(flag_sum * (flag_sum - P::ONES));

    // input bit values in {0, 1}
    for bit in lv.in0 {
        cc.constraint(bit * (bit - P::ONES));
    }
    for bit in lv.in1 {
        cc.constraint(bit * (bit - P::ONES));
    }
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &BitCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
}
