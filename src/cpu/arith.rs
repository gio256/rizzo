use core::borrow::Borrow;
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::stark::Stark;

use crate::cpu::columns::CpuCols;

pub(crate) fn eval<P: PackedField>(
    lv: &CpuCols<P>,
    nv: &CpuCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    let f_alu = lv.op.f_alu;
    let f_imm = lv.f_imm;

    // rd = rs1 + rs2
    // rd = rs1 + imm
    let ch_rs1 = lv.rs1_channel();
    cc.constraint(f_alu * (P::ONES - ch_rs1.f_on));
    cc.constraint(f_alu * ch_rs1.f_rw);
    cc.constraint(f_alu * ch_rs1.adr_seg);
    cc.constraint(f_alu * (lv.rs1_adr() - ch_rs1.adr_virt));

    let ch_rs2 = lv.rs2_channel();
    let use_rs2 = P::ONES - f_imm;
    cc.constraint(f_imm * ch_rs2.f_on);
    cc.constraint(f_alu * use_rs2 * (P::ONES - ch_rs2.f_on));
    cc.constraint(f_alu * use_rs2 * ch_rs2.f_rw);
    cc.constraint(f_alu * use_rs2 * ch_rs2.adr_seg);
    cc.constraint(f_alu * use_rs2 * (lv.rs2_adr() - ch_rs2.adr_virt));

    let ch_rd = lv.rd_channel();
    cc.constraint(f_alu * (P::ONES - ch_rd.f_on));
    cc.constraint(f_alu * (P::ONES - ch_rd.f_rw));
    cc.constraint(f_alu * ch_rd.adr_seg);
    cc.constraint(f_alu * (lv.rd_adr() - ch_rd.adr_virt));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}
