use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::alu::{eval_add_transition, eval_lt};
use crate::cpu::columns::CpuCols;
use crate::cpu::control_flow::INSTRUCTION_BYTES;

pub(crate) fn eval<P: PackedField>(
    lv: &CpuCols<P>,
    nv: &CpuCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    let f_beq = lv.op.f_beq;
    let f_bne = lv.op.f_bne;
    let f_blt = lv.op.f_blt;
    let f_bge = lv.op.f_bge;
    let f_branch = f_beq + f_bne + f_blt + f_bge;
    let f_not_branch = P::ONES - f_branch;

    let blv = lv.shared.branch();
    let f_take_branch = blv.f_take_branch;
    let f_not_take_branch = P::ONES - f_take_branch;

    // f_take_branch in {0, 1}
    cc.constraint(f_branch * f_take_branch * (f_take_branch - P::ONES));

    // disable the rd memory channel
    let ch_rd = lv.rd_channel();
    cc.constraint(f_branch * ch_rd.f_on);

    // read rs1
    let ch_rs1 = lv.rs1_channel();
    cc.constraint(f_branch * (P::ONES - ch_rs1.f_on));
    cc.constraint(f_branch * ch_rs1.f_rw);
    cc.constraint(f_branch * ch_rs1.adr_seg);
    cc.constraint(f_branch * (lv.rs1_adr() - ch_rs1.adr_virt));
    let rs1_val = ch_rs1.val;

    // read rs2
    let ch_rs2 = lv.rs2_channel();
    cc.constraint(f_branch * (P::ONES - ch_rs2.f_on));
    cc.constraint(f_branch * ch_rs2.f_rw);
    cc.constraint(f_branch * ch_rs2.adr_seg);
    cc.constraint(f_branch * (lv.rs2_adr() - ch_rs2.adr_virt));
    let rs2_val = ch_rs2.val;

    //TODO: are these constraints sufficient for blt and bge?
    eval_lt(cc, f_blt, rs1_val, rs2_val, f_take_branch, lv.f_aux1);
    eval_lt(cc, f_bge, rs1_val, rs2_val, f_not_take_branch, lv.f_aux1);

    let diff = rs1_val - rs2_val;
    let diff_pinv = blv.diff_pinv;

    // if beq and branching, rs1_val == rs2_val
    // if beq and rs1_val != rs2_val, not branching
    cc.constraint(f_beq * f_take_branch * diff);
    // if beq and rs1_val == rs2_val, branching
    // if beq and not branching, diff * diff_pinv == 1 (i.e. rs1_val != rs2_val)
    cc.constraint(f_beq * (diff * diff_pinv - f_not_take_branch));

    // if bne and not branching, rs1_val == rs2_val
    // if bne and rs1_val != rs2_val, branching.
    cc.constraint(f_bne * f_not_take_branch * diff);
    // if bne and rs1_val == rs2_val, not branching
    // if bne and branching, rs1_val != rs2_val
    cc.constraint(f_bne * (diff * diff_pinv - f_take_branch));

    // if branching, pc += imm
    eval_add_transition(cc, f_take_branch, lv.pc, lv.imm, nv.pc, lv.f_aux0);

    // if not branching, pc += 4
    let ix_bytes: P = P::Scalar::from_canonical_usize(INSTRUCTION_BYTES).into();
    eval_add_transition(cc, f_not_take_branch, lv.pc, ix_bytes, nv.pc, lv.f_aux0);
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}
