use core::borrow::{Borrow, BorrowMut};
use core::marker::PhantomData;
use core::ops::Deref;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::config::StarkConfig;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::cross_table_lookup::{get_ctl_vars_from_proofs, verify_cross_table_lookups};
use starky::cross_table_lookup::{CrossTableLookup, TableIdx, TableWithColumns};
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter};
use starky::stark::Stark;
use starky::util::trace_rows_to_poly_values;
use starky::verifier::verify_stark_proof_with_challenges;

const N_A_COLS: usize = core::mem::size_of::<ACols<u8>>();
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct ACols<T> {
    pub clk: T,
    pub a: T,
    pub b: T,
}

const N_B_COLS: usize = core::mem::size_of::<BCols<u8>>();
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct BCols<T> {
    pub clk: T,
    pub a: T,
    pub b: T,
}

const N_TABLES: usize = Table::B as usize + 1;
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Table {
    A = 0,
    B = 1,
}
impl Deref for Table {
    type Target = TableIdx;
    fn deref(&self) -> &Self::Target {
        [&0, &1][*self as TableIdx]
    }
}

fn ctl_looking<F: Field>() -> TableWithColumns<F> {
    let cols = ctl_looking_cols::<F>();
    let filter = ctl_looking_filter::<F>();
    TableWithColumns::new(*Table::A, cols, filter)
}

fn ctl_looking_cols<F: Field>() -> Vec<Column<F>> {
    Column::singles([0, 1, 2]).collect()
}

fn ctl_looking_filter<F: Field>() -> Filter<F> {
    let not_col0 = Column::linear_combination_with_constant(vec![(0, F::NEG_ONE)], F::ONE);
    Filter::new_simple(not_col0)
}

fn ctl_looked<F: Field>() -> TableWithColumns<F> {
    let cols = ctl_looked_cols::<F>();
    let filter = ctl_looked_filter::<F>();
    TableWithColumns::new(*Table::B, cols, filter)
}

fn ctl_looked_cols<F: Field>() -> Vec<Column<F>> {
    Column::singles([0, 1, 2]).collect()
}

fn ctl_looked_filter<F: Field>() -> Filter<F> {
    Filter::new_simple(Column::constant(F::ONE))
}

fn ctl<F: Field>() -> CrossTableLookup<F> {
    CrossTableLookup::new(vec![ctl_looking(), ctl_looking()], ctl_looked())
}

impl<F: RichField + Extendable<D>, const D: usize> AStark<F, D> {
    fn trace_row_major() -> Vec<[F; N_A_COLS]> {
        vec![[F::ZERO, F::ZERO, F::ZERO]]
    }
    fn trace() -> Vec<PolynomialValues<F>> {
        let rows = Self::trace_row_major();
        trace_rows_to_poly_values(rows)
    }
}

impl<F: RichField + Extendable<D>, const D: usize> BStark<F, D> {
    fn trace_row_major() -> Vec<[F; N_B_COLS]> {
        vec![[F::ZERO, F::ZERO, F::ZERO], [F::ZERO, F::ZERO, F::ZERO]]
    }
    fn trace() -> Vec<PolynomialValues<F>> {
        let rows = Self::trace_row_major();
        trace_rows_to_poly_values(rows)
    }
}

impl<T: Copy> Borrow<ACols<T>> for [T; N_A_COLS] {
    fn borrow(&self) -> &ACols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<ACols<T>> for [T; N_A_COLS] {
    fn borrow_mut(&mut self) -> &mut ACols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> Borrow<[T; N_A_COLS]> for ACols<T> {
    fn borrow(&self) -> &[T; N_A_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<[T; N_A_COLS]> for ACols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_A_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> Borrow<BCols<T>> for [T; N_B_COLS] {
    fn borrow(&self) -> &BCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<BCols<T>> for [T; N_B_COLS] {
    fn borrow_mut(&mut self) -> &mut BCols<T> {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> Borrow<[T; N_B_COLS]> for BCols<T> {
    fn borrow(&self) -> &[T; N_B_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}
impl<T: Copy> BorrowMut<[T; N_B_COLS]> for BCols<T> {
    fn borrow_mut(&mut self) -> &mut [T; N_B_COLS] {
        unsafe { core::mem::transmute(self) }
    }
}

#[derive(Clone, Copy, Default)]
struct AStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for AStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_A_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_A_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let lv: &[P; N_A_COLS] = frame.get_local_values().try_into().unwrap();
        let lv: &ACols<P> = lv.borrow();
        let nv: &[P; N_A_COLS] = frame.get_next_values().try_into().unwrap();
        let nv: &ACols<P> = nv.borrow();
        todo!()
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let lv: &[ExtensionTarget<D>; N_A_COLS] = frame.get_local_values().try_into().unwrap();
        let lv: &ACols<ExtensionTarget<D>> = lv.borrow();
        let nv: &[ExtensionTarget<D>; N_A_COLS] = frame.get_next_values().try_into().unwrap();
        let nv: &ACols<ExtensionTarget<D>> = nv.borrow();
        todo!()
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}

#[derive(Clone, Copy, Default)]
struct BStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for BStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_B_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_B_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let lv: &[P; N_B_COLS] = frame.get_local_values().try_into().unwrap();
        let lv: &BCols<P> = lv.borrow();
        let nv: &[P; N_B_COLS] = frame.get_next_values().try_into().unwrap();
        let nv: &BCols<P> = nv.borrow();
        todo!()
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let lv: &[ExtensionTarget<D>; N_B_COLS] = frame.get_local_values().try_into().unwrap();
        let lv: &BCols<ExtensionTarget<D>> = lv.borrow();
        let nv: &[ExtensionTarget<D>; N_B_COLS] = frame.get_next_values().try_into().unwrap();
        let nv: &BCols<ExtensionTarget<D>> = nv.borrow();
        todo!()
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashbrown::HashMap;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use starky::cross_table_lookup::debug_utils::check_ctls;

    type F = GoldilocksField;
    const D: usize = 2;

    #[test]
    fn test_ctl() {
        let trace_a = AStark::<F, D>::trace();
        let trace_b = BStark::<F, D>::trace();
        let ctl = ctl::<F>();
        check_ctls(&[trace_a, trace_b], &[ctl], &HashMap::default());
    }
}
