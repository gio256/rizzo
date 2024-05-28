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
use plonky2_maybe_rayon::{MaybeIntoParIter, ParallelIterator};
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::cross_table_lookup::TableWithColumns;
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter, Lookup};
use starky::stark::Stark;

use crate::iter::{windows_mut, LendIter};
use crate::mem::columns::{MemCols, MEM_COL_MAP, N_MEM_COLS};
use crate::mem::Segment;
use crate::stark::Table;

#[derive(Clone, Copy, Debug)]
pub(crate) enum MemKind {
    Read,
    Write,
}

impl From<MemKind> for bool {
    fn from(kind: MemKind) -> bool {
        match kind {
            MemKind::Read => false,
            MemKind::Write => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MemAddress {
    pub seg: Segment,
    pub virt: usize,
}

impl MemAddress {
    pub(crate) fn new(seg: Segment, virt: usize) -> Self {
        Self { seg, virt }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MemOp {
    pub on: bool,
    pub time: usize,
    pub kind: MemKind,
    pub adr: MemAddress,
    pub val: u32,
}

impl MemOp {
    fn sort_key(&self) -> (Segment, usize, usize) {
        (self.adr.seg, self.adr.virt, self.time)
    }

    fn filler(adr: MemAddress, time: usize, val: u32) -> Self {
        Self {
            on: false,
            time,
            kind: MemKind::Read,
            adr,
            val,
        }
    }

    fn into_row<F: Field>(self) -> MemCols<F> {
        let mut row = MemCols::default();
        row.f_on = F::from_bool(self.on);
        row.f_rw = F::from_bool(self.kind.into());
        row.time = F::from_canonical_usize(self.time);
        row.adr_seg = F::from_canonical_usize(self.adr.seg as usize);
        row.adr_virt = F::from_canonical_usize(self.adr.virt);
        row.val = F::from_canonical_u32(self.val);
        row
    }
}

pub(crate) fn gen_trace<F: RichField>(mut ops: Vec<MemOp>) -> Vec<MemCols<F>> {
    ops.sort_by_key(MemOp::sort_key);

    // fill range check gaps, then sort again and add padding rows
    fill_rc_gaps(&mut ops);
    ops.sort_by_key(MemOp::sort_key);
    pad(&mut ops);

    let mut rc_freq = HashMap::default();
    let mut rows = ops
        .into_par_iter()
        .map(MemOp::into_row::<F>)
        .collect::<Vec<_>>();
    let mut iter = windows_mut::<_, 2>(&mut rows);
    while let Some([lv, nv]) = iter.next() {
        trace(lv, nv, &mut rc_freq);
    }

    for (val, freq) in rc_freq {
        let idx: usize = val.to_canonical_u64().try_into().unwrap();
        rows[idx].rc_freq = F::from_canonical_usize(freq);
    }
    rows
}

fn trace<F: RichField>(lv: &mut MemCols<F>, nv: &mut MemCols<F>, map: &mut HashMap<F, usize>) {
    let seg_diff = lv.adr_seg != nv.adr_seg;
    let virt_diff = lv.adr_virt != nv.adr_virt && !seg_diff;

    lv.f_seg_diff = F::from_bool(seg_diff);
    lv.f_virt_diff = F::from_bool(virt_diff);

    let reg0 = lv.adr_seg == F::ZERO && lv.adr_virt == F::ZERO;
    lv.f_reg0 = F::from_bool(reg0);

    let aux = !(seg_diff || virt_diff || reg0);
    lv.aux = F::from_bool(aux);

    // range checks
    lv.rc = if seg_diff {
        nv.adr_seg - lv.adr_seg - F::ONE
    } else if virt_diff {
        nv.adr_virt - lv.adr_virt - F::ONE
    } else {
        nv.time - lv.time
    };
    nv.rc_count = lv.rc_count + F::ONE;
    let freq = map.entry(lv.rc).or_insert(0);
    *freq += 1;

    if seg_diff {
        let freq = map.entry(nv.adr_virt).or_insert(0);
        *freq += 1;
    }
}

fn pad(ops: &mut Vec<MemOp>) {
    let last_op = *ops.last().unwrap();
    let pad_op = MemOp {
        on: false,
        kind: MemKind::Read,
        ..last_op
    };
    let len = ops.len();
    let padded_len = len.next_power_of_two();
    for _ in len..padded_len {
        ops.push(pad_op)
    }
}

fn fill_rc_gaps(ops: &mut Vec<MemOp>) {
    let max_rc = ops.len().next_power_of_two() - 1;
    let fill_ops = ops
        .array_windows::<2>()
        .flat_map(|[lv, nv]| fill_gap(lv, nv, max_rc))
        .collect::<Vec<_>>();
    ops.extend(fill_ops);
}

fn fill_gap<'a>(lv: &'a MemOp, nv: &'a MemOp, max_rc: usize) -> impl Iterator<Item = MemOp> + 'a {
    // a hack to allow returning different concrete iterators from each branch
    let mut res_a = None;
    let mut res_b = None;
    let mut res_c = None;

    if lv.adr.seg != nv.adr.seg {
        let gap = nv.adr.virt / max_rc;
        let res = (1..gap + 1).map(move |i| {
            let adr = MemAddress::new(lv.adr.seg, lv.adr.virt - max_rc * i);
            MemOp::filler(adr, 0, 0)
        });
        res_a = Some(res);
    } else if lv.adr.virt != nv.adr.virt {
        let gap = (nv.adr.virt - lv.adr.virt - 1) / max_rc;
        let res = (1..gap + 1).map(move |i| {
            let adr = MemAddress::new(lv.adr.seg, lv.adr.virt + (max_rc + 1) * i);
            MemOp::filler(adr, 0, 0)
        });
        res_b = Some(res);
    } else {
        let gap = (nv.time - lv.time) / max_rc;
        let res = (1..gap + 1).map(move |i| MemOp::filler(lv.adr, lv.time + max_rc * i, lv.val));
        res_c = Some(res);
    }

    res_a
        .into_iter()
        .flatten()
        .chain(res_b.into_iter().flatten())
        .chain(res_c.into_iter().flatten())
}
