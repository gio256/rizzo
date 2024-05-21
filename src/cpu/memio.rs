use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::alu::eval_add;
use crate::cpu::columns::CpuCols;

fn eval_load<P: PackedField>(lv: &CpuCols<P>, nv: &CpuCols<P>, cc: &mut ConstraintConsumer<P>) {
    // rd = M[rs1+imm]
    let f_lw = lv.op.f_lw;

    // read rs1
    let ch_rs1 = lv.rs1_channel();
    cc.constraint(f_lw * (P::ONES - ch_rs1.f_on));
    cc.constraint(f_lw * ch_rs1.f_rw);
    cc.constraint(f_lw * ch_rs1.adr_seg);
    cc.constraint(f_lw * (lv.rs1_adr() - ch_rs1.adr_virt));

    // write loaded value to rd
    let ch_rd = lv.rd_channel();
    cc.constraint(f_lw * (P::ONES - ch_rd.f_on));
    cc.constraint(f_lw * (P::ONES - ch_rd.f_rw));
    cc.constraint(f_lw * ch_rd.adr_seg);
    cc.constraint(f_lw * (lv.rd_adr() - ch_rd.adr_virt));

    // use rs2 channel to load from memory
    let ch_load = lv.rs2_channel();
    cc.constraint(f_lw * (P::ONES - ch_load.f_on));
    cc.constraint(f_lw * ch_load.f_rw);
    cc.constraint(f_lw * (P::ONES - ch_load.adr_seg));
    cc.constraint(f_lw * (ch_rd.val - ch_load.val));
    eval_add(cc, f_lw, ch_rs1.val, lv.imm, ch_load.adr_virt, lv.f_aux1);
}

fn eval_store<P: PackedField>(lv: &CpuCols<P>, nv: &CpuCols<P>, cc: &mut ConstraintConsumer<P>) {
    // M[rs1+imm] = rs2
    let f_sw = lv.op.f_sw;

    // read rs1
    let ch_rs1 = lv.rs1_channel();
    cc.constraint(f_sw * (P::ONES - ch_rs1.f_on));
    cc.constraint(f_sw * ch_rs1.f_rw);
    cc.constraint(f_sw * ch_rs1.adr_seg);
    cc.constraint(f_sw * (lv.rs1_adr() - ch_rs1.adr_virt));

    // read rs2
    let ch_rs2 = lv.rs2_channel();
    cc.constraint(f_sw * (P::ONES - ch_rs2.f_on));
    cc.constraint(f_sw * ch_rs2.f_rw);
    cc.constraint(f_sw * ch_rs2.adr_seg);
    cc.constraint(f_sw * (lv.rs2_adr() - ch_rs2.adr_virt));

    // use rd channel to write to memory
    let ch_store = lv.rd_channel();
    cc.constraint(f_sw * (P::ONES - ch_store.f_on));
    cc.constraint(f_sw * (P::ONES - ch_store.f_rw));
    cc.constraint(f_sw * (P::ONES - ch_store.adr_seg));
    cc.constraint(f_sw * (ch_rs2.val - ch_store.val));
    eval_add(cc, f_sw, ch_rs1.val, lv.imm, ch_store.adr_virt, lv.f_aux1);
}

pub(crate) fn eval<P: PackedField>(
    lv: &CpuCols<P>,
    nv: &CpuCols<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    eval_load(lv, nv, cc);
    eval_store(lv, nv, cc);
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}
