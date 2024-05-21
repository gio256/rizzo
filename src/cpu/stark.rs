use core::borrow::Borrow;
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::cross_table_lookup::TableWithColumns;
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter, Lookup};
use starky::stark::Stark;

use crate::cpu::columns::{CpuCols, CPU_COL_MAP, N_CPU_COLS, N_MEM_CHANNELS};
use crate::cpu::{arith, branch, clock, control_flow, decode, flags, jump, membus, memio, reg};
use crate::stark::Table;

fn mem_timestamp<F: Field>(channel: usize) -> Column<F> {
    let n = F::from_canonical_usize(N_MEM_CHANNELS);
    let ch = F::from_canonical_usize(channel);
    Column::linear_combination_with_constant([(CPU_COL_MAP.clock, n)], ch)
}

pub(crate) fn ctl_looking_mem<F: Field>(channel: usize) -> TableWithColumns<F> {
    let ch = CPU_COL_MAP.membus[channel];
    let mut cols: Vec<_> = Column::singles([ch.f_rw, ch.adr_seg, ch.adr_virt, ch.val]).collect();
    cols.push(mem_timestamp(channel));

    let filter = Filter::new_simple(Column::single(CPU_COL_MAP.membus[channel].f_on));
    TableWithColumns::new(Table::Cpu as usize, cols, filter)
}

pub(crate) fn ctl_looking_alu_reg<F: Field>() -> TableWithColumns<F> {
    let cols = Column::singles([
        CPU_COL_MAP.opcode,
        CPU_COL_MAP.rs1_channel().val,
        CPU_COL_MAP.rs2_channel().val,
        CPU_COL_MAP.rd_channel().val,
    ])
    .collect();

    let f_not_imm =
        Column::linear_combination_with_constant(vec![(CPU_COL_MAP.f_imm, F::NEG_ONE)], F::ONE);
    let f_alu = Column::single(CPU_COL_MAP.op.f_alu);
    let filter = Filter::new(vec![(f_not_imm, f_alu)], vec![]);

    TableWithColumns::new(Table::Cpu as usize, cols, filter)
}

pub(crate) fn ctl_looking_alu_imm<F: Field>() -> TableWithColumns<F> {
    let cols = Column::singles([
        CPU_COL_MAP.opcode,
        CPU_COL_MAP.rs1_channel().val,
        CPU_COL_MAP.imm,
        CPU_COL_MAP.rd_channel().val,
    ])
    .collect();

    let f_imm = Column::single(CPU_COL_MAP.f_imm);
    let f_alu = Column::single(CPU_COL_MAP.op.f_alu);
    let filter = Filter::new(vec![(f_imm, f_alu)], vec![]);

    TableWithColumns::new(Table::Cpu as usize, cols, filter)
}

fn eval_all<P: PackedField>(lv: &CpuCols<P>, nv: &CpuCols<P>, cc: &mut ConstraintConsumer<P>) {
    clock::eval(lv, nv, cc);
    control_flow::eval(lv, nv, cc);
    membus::eval(lv, nv, cc);
    memio::eval(lv, nv, cc);
    decode::eval(lv, nv, cc);
    jump::eval(lv, nv, cc);
    branch::eval(lv, nv, cc);
    flags::eval(lv, nv, cc);
    arith::eval(lv, nv, cc);
    reg::eval(lv, nv, cc);
}

fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &CpuCols<ExtensionTarget<D>>,
    nv: &CpuCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    clock::eval_circuit(cb, lv, nv, cc);
    control_flow::eval_circuit(cb, lv, nv, cc);
    membus::eval_circuit(cb, lv, nv, cc);
    memio::eval_circuit(cb, lv, nv, cc);
    decode::eval_circuit(cb, lv, nv, cc);
    jump::eval_circuit(cb, lv, nv, cc);
    branch::eval_circuit(cb, lv, nv, cc);
    flags::eval_circuit(cb, lv, nv, cc);
    arith::eval_circuit(cb, lv, nv, cc);
    reg::eval_circuit(cb, lv, nv, cc);
}

#[derive(Clone, Copy, Default)]
pub struct CpuStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for CpuStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_CPU_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_CPU_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_CPU_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &CpuCols<P> = local.borrow();
        let next: &[P; N_CPU_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &CpuCols<P> = next.borrow();

        eval_all(local, next, cc)
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_CPU_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &CpuCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_CPU_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &CpuCols<ExtensionTarget<D>> = next.borrow();

        eval_all_circuit(cb, local, next, cc);
    }

    fn constraint_degree(&self) -> usize {
        3
    }

    fn requires_ctls(&self) -> bool {
        true
    }
}
