use core::borrow::Borrow;
use core::cmp::max;
use core::iter::once;
use core::marker::PhantomData;

use hashbrown::HashMap;
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

use crate::iter::{windows_mut, LendIter};
use crate::pack::columns::{PackCols, N_PACK_COLS, PACK_COL_MAP};
use crate::pack::N_BYTES;
use crate::stark::Table;

pub(crate) struct PackOp {
    pub rw: bool,
    pub adr_virt: u32,
    pub time: u32,
    pub bytes: Vec<u8>,
}

pub(crate) fn gen_trace<F: RichField>(ops: Vec<PackOp>, min_rows: usize) -> Vec<PackCols<F>> {
    let ops_len = ops.iter().map(|op| usize::from(!op.bytes.is_empty())).sum();
    let n_rows = max(max(ops_len, u8::MAX.into()), min_rows).next_power_of_two();
    let mut rows: Vec<PackCols<F>> = vec![Default::default(); n_rows];

    let window = windows_mut::<_, 2>(&mut rows);
    let mut iter = window.zip(ops.into_iter().filter(|op| !op.bytes.is_empty()));
    let mut rc_freq = HashMap::default();

    // padding rows are empty
    while let Some(([lv, nv], op)) = iter.next() {
        trace(lv, nv, &mut rc_freq, op);
    }

    for (val, freq) in rc_freq {
        rows[val as usize].rc_freq = F::from_canonical_usize(freq);
    }
    rows
}

pub(crate) fn trace<F: RichField>(
    lv: &mut PackCols<F>,
    nv: &mut PackCols<F>,
    map: &mut HashMap<u8, usize>,
    op: PackOp,
) {
    let len = op.bytes.len();
    debug_assert!(len > 0 && len <= N_BYTES);
    lv.f_rw = F::from_bool(op.rw);
    lv.adr_virt = F::from_canonical_u32(op.adr_virt);
    lv.time = F::from_canonical_u32(op.time);
    lv.len_idx[len - 1] = F::ONE;
    lv.bytes = op
        .bytes
        .into_iter()
        .rev()
        .map(|b| {
            let freq = map.entry(b).or_insert(0);
            *freq += 1;
            F::from_canonical_u8(b)
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    if lv.rc_count.to_canonical_u64() < u8::MAX.into() {
        nv.rc_count = lv.rc_count + F::ONE;
    } else {
        nv.rc_count = F::from_canonical_u8(u8::MAX);
    }
}
