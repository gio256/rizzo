use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::cpu::columns::CpuCols;

pub(crate) fn eval<P: PackedField>(
    lv: &CpuCols<P>,
    nv: &CpuCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    // The clock starts at zero.
    cc.constraint_first_row(lv.clock);
    // Each row increments the clock by one.
    cc.constraint_transition(nv.clock - lv.clock - P::ONES);
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    // The clock starts at zero.
    cc.constraint_first_row(cb, lv.clock);

    // Each row increments the clock by one.
    let new_clock = cb.add_const_extension(lv.clock, F::ONE);
    let cs = cb.sub_extension(nv.clock, new_clock);
    cc.constraint_transition(cb, cs);
}
