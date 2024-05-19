use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::cpu::arith::eval_add;
use crate::cpu::columns::{CpuCols, CPU_COL_MAP};

const INC_PC_OPS: [usize; 2] = [CPU_COL_MAP.op.f_alu, CPU_COL_MAP.op.f_lb];
pub(crate) const INSTRUCTION_BYTES: usize = 4;

pub(crate) fn eval<P: PackedField>(
    lv: &CpuCols<P>,
    nv: &CpuCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    let is_op: P = CPU_COL_MAP.op.iter().map(|&i| lv[i]).sum();
    let is_op_next: P = CPU_COL_MAP.op.iter().map(|&i| nv[i]).sum();
    let halt_next = P::ONES - is_op_next;

    cc.constraint_transition(is_op * (is_op_next + halt_next - P::ONES));

    let f_inc_pc: P = INC_PC_OPS.iter().map(|&i| lv[i]).sum();
    let ix_bytes: P = P::Scalar::from_canonical_usize(INSTRUCTION_BYTES).into();
    eval_add(cc, f_inc_pc, lv.pc, ix_bytes, nv.pc, lv.f_aux0);
    // cc.constraint_transition(inc_pc * (nv.pc - lv.pc - P::Scalar::from_canonical_u8(4)));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}
