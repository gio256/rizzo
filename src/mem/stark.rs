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

use crate::mem::columns::{MemCols, MEM_COL_MAP, N_MEM_COLS};
use crate::stark::Table;

pub(crate) fn ctl_looked<F: Field>() -> TableWithColumns<F> {
    let cols = Column::singles([
        MEM_COL_MAP.f_rw,
        MEM_COL_MAP.adr_seg,
        MEM_COL_MAP.adr_virt,
        MEM_COL_MAP.val,
        MEM_COL_MAP.time,
    ])
    .collect();

    let filter = Filter::new_simple(Column::single(MEM_COL_MAP.f_on));
    TableWithColumns::new(Table::Mem as usize, cols, filter)
}

fn eval_all<P: PackedField>(lv: &MemCols<P>, nv: &MemCols<P>, cc: &mut ConstraintConsumer<P>) {
    // f_on in {0, 1}
    let f_on = lv.f_on;
    let f_off = P::ONES - f_on;
    cc.constraint(f_on * f_off);

    // f_rw in {0, 1} is enforced by CTL
    let f_read = P::ONES - lv.f_rw;
    let f_read_next = P::ONES - nv.f_rw;

    // padding rows must be reads
    cc.constraint(f_off * lv.f_rw);

    // local values
    let adr_seg = lv.adr_seg;
    let adr_virt = lv.adr_virt;
    let val = lv.val;

    // next values
    let adr_seg_next = nv.adr_seg;
    let adr_virt_next = nv.adr_virt;
    let val_next = nv.val;

    // flags
    let f_reg0 = lv.f_reg0;
    let f_not_reg0 = P::ONES - f_reg0;
    let f_seg_diff = lv.f_seg_diff;
    let f_virt_diff = lv.f_virt_diff;
    let f_seg_same = P::ONES - f_seg_diff;
    let f_virt_same = P::ONES - f_virt_diff;
    let f_adr_diff = f_seg_diff + f_virt_diff;
    let f_adr_same = P::ONES - f_adr_diff;

    // flags in {0, 1}
    cc.constraint(f_reg0 * f_not_reg0);
    cc.constraint(f_seg_diff * f_seg_same);
    cc.constraint(f_virt_diff * f_virt_same);
    // at most one diff flag should be set
    cc.constraint(f_adr_diff * f_adr_same);

    // no change before change flag
    cc.constraint(f_virt_diff * (adr_seg_next - adr_seg));
    cc.constraint(f_adr_same * (adr_seg_next - adr_seg));
    cc.constraint(f_adr_same * (adr_virt_next - adr_virt));

    let range_check = f_seg_diff * (adr_seg_next - adr_seg - P::ONES)
        + f_virt_diff * (adr_virt_next - adr_virt - P::ONES)
        + f_adr_same * (nv.time - lv.time);
    cc.constraint(lv.rc - range_check);

    // reads keep the same value as the current row, except for register x0
    // f_read_next * f_adr_same * f_not_reg0 * (val_next - val);
    let aux = lv.aux;
    cc.constraint(aux - f_adr_same * f_not_reg0);
    cc.constraint_transition(f_read_next * aux * (val_next - val));

    // all memory is initialized to 0
    cc.constraint_transition(f_read_next * f_adr_diff * val_next);

    // register x0 is always 0
    cc.constraint(f_reg0 * adr_seg);
    cc.constraint(f_reg0 * adr_virt);
    cc.constraint(f_read * f_reg0 * val);

    // rc_count starts at 0 and increments by 1
    cc.constraint_first_row(lv.rc_count);
    cc.constraint_transition(nv.rc_count - lv.rc_count - P::ONES);
}

fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &MemCols<ExtensionTarget<D>>,
    nv: &MemCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    let one = cb.one_extension();

    let f_on = lv.f_on;
    let f_not_on = cb.sub_extension(f_on, one);
    let cs = cb.mul_extension(f_on, f_not_on);
    cc.constraint(cb, cs);

    let f_rw = lv.f_rw;

    let f_off = cb.sub_extension(one, f_on);
    let cs = cb.mul_extension(f_off, f_rw);
    cc.constraint(cb, cs);
}

#[derive(Clone, Copy, Default)]
pub struct MemStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for MemStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_MEM_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget = StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_MEM_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_MEM_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &MemCols<P> = local.borrow();
        let next: &[P; N_MEM_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &MemCols<P> = next.borrow();
        eval_all(local, next, cc)
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_MEM_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &MemCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_MEM_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &MemCols<ExtensionTarget<D>> = next.borrow();
        eval_all_circuit(cb, local, next, cc);
    }

    fn constraint_degree(&self) -> usize {
        3
    }

    fn lookups(&self) -> Vec<Lookup<F>> {
        vec![Lookup {
            columns: vec![
                Column::single(MEM_COL_MAP.rc),
                Column::single_next_row(MEM_COL_MAP.adr_virt),
            ],
            table_column: Column::single(MEM_COL_MAP.rc_count),
            frequencies_column: Column::single(MEM_COL_MAP.rc_freq),
            filter_columns: vec![
                Default::default(),
                Filter::new_simple(Column::single(MEM_COL_MAP.f_seg_diff)),
            ],
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

    use super::MemStark;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type S = MemStark<F, D>;

    #[test]
    fn test_stark_degree() {
        let stark: S = Default::default();
        test_stark_low_degree(stark).unwrap();
    }

    // #[test]
    // fn test_stark_circuit() {
    //     let stark: S = Default::default();
    //     test_stark_circuit_constraints::<F, C, S, D>(stark).unwrap();
    // }
}
