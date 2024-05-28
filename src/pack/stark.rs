use core::borrow::Borrow;
use core::marker::PhantomData;
use std::iter::once;

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

use crate::pack::columns::{PackCols, N_PACK_COLS, PACK_COL_MAP};
use crate::pack::N_BYTES;
use crate::stark::Table;

pub(crate) fn ctl_looked<F: Field>() -> TableWithColumns<F> {
    let len_comb = PACK_COL_MAP
        .len_idx
        .into_iter()
        .enumerate()
        .map(|(i, col)| (col, F::from_canonical_usize(i + 1)));
    let len = Column::linear_combination(len_comb);

    let packed_comb = PACK_COL_MAP
        .bytes
        .into_iter()
        .enumerate()
        .map(|(i, col)| (col, F::from_canonical_u64(1 << (8 * i))));
    let packed = Column::linear_combination(packed_comb);

    let f_rw = Column::single(PACK_COL_MAP.f_rw);
    let adr_virt = Column::single(PACK_COL_MAP.adr_virt);
    let time = Column::single(PACK_COL_MAP.time);

    let cols = vec![f_rw, adr_virt, len, packed, time];
    let filter = Filter::new_simple(Column::sum(PACK_COL_MAP.len_idx));
    TableWithColumns::new(Table::Pack as usize, cols, filter)
}

pub(crate) fn ctl_looking_mem<F: Field>(i: usize) -> TableWithColumns<F> {
    // `virtual_address_col = adr_virt + len - 1 - i`
    let len_sub1_comb = PACK_COL_MAP
        .len_idx
        .into_iter()
        .enumerate()
        .map(|(i, col)| (col, F::from_canonical_usize(i)));
    let adr_virt_comb = once((PACK_COL_MAP.adr_virt, F::ONE)).chain(len_sub1_comb);
    let adr_virt = Column::linear_combination_with_constant(
        adr_virt_comb,
        F::NEG_ONE * F::from_canonical_usize(i),
    );

    let f_rw = Column::single(PACK_COL_MAP.f_rw);
    let adr_seg = Column::constant(F::ONE);
    let byte = Column::single(PACK_COL_MAP.bytes[i]);
    let time = Column::single(PACK_COL_MAP.time);

    let cols = vec![f_rw, adr_seg, adr_virt, byte, time];
    let filter = Filter::new_simple(Column::sum(&PACK_COL_MAP.len_idx[i..]));
    TableWithColumns::new(Table::Pack as usize, cols, filter)
}

fn eval_all<P: PackedField>(lv: &PackCols<P>, nv: &PackCols<P>, cc: &mut ConstraintConsumer<P>) {
    // filter in {0, 1} and starts at 1
    let filter: P = lv.len_idx.into_iter().sum();
    cc.constraint(filter * (filter - P::ONES));
    cc.constraint_first_row(filter - P::ONES);

    // len_idx values in {0, 1}
    for idx in lv.len_idx {
        cc.constraint(idx * (idx - P::ONES));
    }

    // f_rw in {0, 1}
    let f_rw = lv.f_rw;
    cc.constraint(f_rw * (f_rw - P::ONES));

    // all bytes beyond the length must be 0
    for (i, idx) in lv.len_idx.into_iter().enumerate() {
        for &byte in &lv.bytes[i + 1..] {
            cc.constraint(idx * byte);
        }
    }

    // all filters are on until padding starts
    let filter_next: P = nv.len_idx.into_iter().sum();
    cc.constraint_transition(filter_next * (filter_next - filter));

    // range check
    let count = lv.rc_count;
    let count_next = nv.rc_count;
    let delta = count_next - count;
    cc.constraint_first_row(count);
    cc.constraint_transition(delta * (delta - P::ONES));
    cc.constraint_last_row(count - P::Scalar::from_canonical_u8(u8::MAX));
}

fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &PackCols<ExtensionTarget<D>>,
    nv: &PackCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    todo!()
}

#[derive(Clone, Copy, Default)]
pub struct PackStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for PackStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_PACK_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_PACK_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_PACK_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &PackCols<P> = local.borrow();
        let next: &[P; N_PACK_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &PackCols<P> = next.borrow();
        eval_all(local, next, cc)
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_PACK_COLS] =
            frame.get_local_values().try_into().unwrap();
        let local: &PackCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_PACK_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &PackCols<ExtensionTarget<D>> = next.borrow();
        eval_all_circuit(cb, local, next, cc);
    }

    fn constraint_degree(&self) -> usize {
        3
    }

    fn lookups(&self) -> Vec<Lookup<F>> {
        vec![Lookup {
            columns: Column::singles(PACK_COL_MAP.bytes).collect(),
            table_column: Column::single(PACK_COL_MAP.rc_count),
            frequencies_column: Column::single(PACK_COL_MAP.rc_freq),
            filter_columns: vec![Default::default(); N_BYTES],
        }]
    }

    fn requires_ctls(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use starky::stark_testing::{test_stark_circuit_constraints, test_stark_low_degree};

    use super::PackStark;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type S = PackStark<F, D>;

    #[test]
    fn stark_degree() {
        let stark: S = Default::default();
        test_stark_low_degree(stark).unwrap();
    }

    // #[test]
    // fn stark_circuit() {
    //     let stark: S = Default::default();
    //     test_stark_circuit_constraints::<F, C, S, D>(stark).unwrap();
    // }
}
