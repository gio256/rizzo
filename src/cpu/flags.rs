use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::cpu::columns::{CpuCols, CPU_COL_MAP};

pub(crate) fn eval<P: PackedField>(
    lv: &CpuCols<P>,
    nv: &CpuCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    cc.constraint(lv.f_imm * (lv.f_imm - P::ONES));
    cc.constraint(lv.f_aux0 * (lv.f_aux0 - P::ONES));
    cc.constraint(lv.f_aux1 * (lv.f_aux1 - P::ONES));

    for i in *CPU_COL_MAP.op {
        let flag = lv[i];
        cc.constraint(flag * (flag - P::ONES));
    }
    let flag_sum: P = CPU_COL_MAP.op.iter().map(|&i| lv[i]).sum();
    cc.constraint(flag_sum * (flag_sum - P::ONES));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}
