use core::borrow::{Borrow, BorrowMut};
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::util::timing::TimingTree;
use starky::config::StarkConfig;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::stark::Stark;
use starky::util::trace_rows_to_poly_values;

use crate::columns::{CpuColumns, OpColumns, N_COLUMNS};

pub const N_REGISTERS: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct VmStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

pub struct Registers {}

pub struct FmtR {
    rd: usize,
    rs1: usize,
    rs2: usize,
}
pub enum OpR {
    Add,
    Sub,
    Xor,
    Or,
    And,
    Sll,
    Srl,
    Sra,
    Slt,
    Sltu,
}
pub struct IxR {
    op: OpR,
    fmt: FmtR,
}

#[derive(Clone, Debug)]
pub struct Vm {
    pub ops: Vec<Instruction>,
    pub pc: u32,
    pub reg: [u32; N_REGISTERS],
}

impl Vm {
    pub fn new(ops: Vec<Instruction>) -> Self {
        Self {
            ops,
            pc: 0,
            reg: [0; 32],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Add,
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub op: Op,
    pub rd: Option<u64>,
    pub rs1: Option<u64>,
    pub rs2: Option<u64>,
    pub imm: Option<u32>,
}

pub fn op_to_row<F: PackedField>(row: &mut OpColumns<F>, op: Op) {
    match op {
        Op::Add => row.is_add = F::ONES,
    }
}

fn trace<F: RichField + Extendable<D>, const D: usize>(
    cfg: StarkConfig,
    t: &mut TimingTree,
) -> Vec<PolynomialValues<F>> {
    let mut row = [F::ZERO; N_COLUMNS];
    let cols: &mut CpuColumns<F> = row.borrow_mut();
    cols.op.is_add = F::ONES;
    let rows = vec![row];
    trace_rows_to_poly_values(rows)
}

fn eval_packed<P: PackedField>(
    lv: &CpuColumns<P>,
    nv: &CpuColumns<P>,
    cc: &mut ConstraintConsumer<P>,
) {
    cc.constraint_first_row(lv.clk);
    cc.constraint_transition(nv.clk - lv.clk - P::ONES);

    // cc.constraint_transition(nv.pc - lv.pc - P::from_canonnical_u8(4));
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for VmStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_COLUMNS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_COLUMNS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_COLUMNS] = frame.get_local_values().try_into().unwrap();
        let local: &CpuColumns<P> = local.borrow();
        let next: &[P; N_COLUMNS] = frame.get_next_values().try_into().unwrap();
        let next: &CpuColumns<P> = next.borrow();
        eval_packed(local, next, cc)
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        todo!()
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}
