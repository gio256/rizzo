use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::cpu::columns::CpuCols;
use crate::cpu::arith::eval_add;
use crate::cpu::control_flow::INSTRUCTION_BYTES;

pub(crate) fn eval<P: PackedField>(
    lv: &CpuCols<P>,
    nv: &CpuCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    let f_jal = lv.op.f_jal;
    let f_jalr = lv.op.f_jalr;
    let f_jump = f_jal + f_jalr;

    // jal sets PC = PC + imm
    eval_add(cc, f_jal, lv.pc, lv.imm, nv.pc, lv.f_aux0);

    // jalr sets PC = rs1 + imm
    let ch_rs1 = lv.rs1_channel();
    cc.constraint(f_jalr * (P::ONES - ch_rs1.f_on));
    cc.constraint(f_jalr * ch_rs1.f_rw);
    cc.constraint(f_jalr * ch_rs1.adr_seg);
    cc.constraint(f_jalr * (lv.rs1 - ch_rs1.adr_virt));
    eval_add(cc, f_jalr, lv.pc, ch_rs1.val, nv.pc, lv.f_aux0);

    // jal disables the rs1 memory channel
    cc.constraint(f_jal * (P::ONES - ch_rs1.f_on));

    // both jal and jalr set rd = PC + 4
    let ch_rd = lv.rd_channel();
    cc.constraint(f_jump * (P::ONES - ch_rd.f_on));
    cc.constraint(f_jump * (P::ONES - ch_rd.f_rw));
    cc.constraint(f_jump * ch_rd.adr_seg);
    cc.constraint(f_jump * (lv.rd - ch_rd.adr_virt));
    let ix_bytes: P = P::Scalar::from_canonical_usize(INSTRUCTION_BYTES).into();
    eval_add(cc, f_jump, lv.pc, ix_bytes, ch_rd.val, lv.f_aux1);

    // both jal and jalr disable the rs2 memory channel
    let ch_rs2 = lv.rs2_channel();
    cc.constraint(f_jump * (P::ONES - ch_rs2.f_on));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}
