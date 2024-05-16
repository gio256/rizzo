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
    let f_jal = lv.op.f_jal;
    let f_jalr = lv.op.f_jalr;

    // cc.constraint_transition(f_jalr * (nv.pc - lv.membus[0].val - lv.imm));
    // cc.constraint_transition(f_jalr * (nv.pc - lv.op1 - lv.op2));
    // let rd_val = lv.membus[N_MEM_CHANNELS - 1].val;
    // cc.constraint(f_jal * (rd_val - lv.pc - P::Scalar::from_canonical_u8(4)));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}
