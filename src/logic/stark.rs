use core::borrow::Borrow;
use core::iter::zip;
use core::marker::PhantomData;

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::types::{Field, PrimeField64};
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use starky::cross_table_lookup::TableWithColumns;
use starky::evaluation_frame::{StarkEvaluationFrame, StarkFrame};
use starky::lookup::{Column, Filter};
use starky::stark::Stark;

use crate::logic::columns::{LogicCols, LOGIC_COL_MAP, N_LOGIC_COLS, WORD_BITS};
use crate::stark::Table;
use crate::util::fst;
use crate::vm::opcode::Opcode;

/// Operation flags and the corresponding opcode for AND, XOR, and OR.
const LOGIC_OPS: [(usize, u8); 3] = [
    (LOGIC_COL_MAP.op.f_and, Opcode::AND as u8),
    (LOGIC_COL_MAP.op.f_xor, Opcode::XOR as u8),
    (LOGIC_COL_MAP.op.f_or, Opcode::OR as u8),
];

/// Operation flags and the corresponding opcode for SLL, SRL, and SRA.
const SHIFT_OPS: [(usize, u8); 3] = [
    (LOGIC_COL_MAP.op.f_sll, Opcode::SLL as u8),
    (LOGIC_COL_MAP.op.f_srl, Opcode::SRL as u8),
    (LOGIC_COL_MAP.op.f_sra, Opcode::SRA as u8),
];

pub(crate) fn ctl_looked_logic<F: Field>() -> TableWithColumns<F> {
    let op_comb = LOGIC_OPS.map(|(f, op)| (f, F::from_canonical_u8(op)));
    let op = Column::linear_combination(op_comb);
    let in0 = Column::le_bits(LOGIC_COL_MAP.in0);
    let in1 = Column::le_bits(LOGIC_COL_MAP.in1);
    let out = Column::single(LOGIC_COL_MAP.out);

    let cols = vec![op, in0, in1, out];
    let filter = Filter::new_simple(Column::sum(LOGIC_OPS.map(fst)));
    TableWithColumns::new(Table::Logic as usize, cols, filter)
}

pub(crate) fn ctl_looked_shift<F: Field>() -> TableWithColumns<F> {
    let op_comb = SHIFT_OPS.map(|(f, op)| (f, F::from_canonical_u8(op)));
    let op = Column::linear_combination(op_comb);
    let in0 = Column::le_bits(LOGIC_COL_MAP.in0);
    let shift_amt_comb = LOGIC_COL_MAP
        .in1
        .into_iter()
        .enumerate()
        .map(|(i, col)| (col, F::from_canonical_usize(i)));
    let in1 = Column::linear_combination(shift_amt_comb);
    let out = Column::single(LOGIC_COL_MAP.out);

    let cols = vec![op, in0, in1, out];
    let filter = Filter::new_simple(Column::sum(SHIFT_OPS.map(fst)));
    TableWithColumns::new(Table::Logic as usize, cols, filter)
}

/// Logical shift towards the most significant bit.
/// [ 1 0 0 0 1 0 0 0 ] input bits (LE)
/// [ 0 0 0 1 0 0 0 0 ] shift_amt, indexed by n
/// [ 0 0 0 1 0 0 0 1 ] output
///```ignore
/// let mut sll_out = P::ZEROS;
/// for (n, f_n) in lv.in1.into_iter().enumerate() {
///     for m in n..WORD_BITS {
///         sll_out += f_n * lv.in0[m - n] * P::Scalar::from_canonical_u32(1 << m);
///     }
/// }
///```
/// n is the (hypothetical) number of bits to shift by.
/// f_n is a flag indicating whether this is really the n to shift by.
/// All bits in the output with index < n are 0.
/// We match up bits[..WORD_BITS - n] with the sequence (2^n, 2^n+1, ...).
fn sll<P: PackedField>(bits: &[P; WORD_BITS], shift_amt: &[P; WORD_BITS]) -> P {
    shift_amt
        .iter()
        .enumerate()
        .flat_map(|(n, &f_n)| {
            bits.iter()
                .take(WORD_BITS - n)
                .zip(P::Scalar::TWO.powers().skip(n))
                .map(move |(&bit, base)| f_n * bit * base)
        })
        .sum()
}

/// Logical shift towards the least significant bit.
///[ 0 0 0 1 0 0 0 1 ] input bits (LE)
///[ 0 0 0 1 0 0 0 0 ] shift_amt, indexed by n
///[ 1 0 0 0 1 0 0 0 ] output
///```ignore
/// let mut srl_out = P::ZEROS;
/// for (n, f_n) in lv.in1.into_iter().enumerate() {
///     for m in n..WORD_BITS {
///         srl_out += f_n * lv.in0[m] * P::Scalar::from_canonical_u32(1 << (m - n));
///     }
/// }
///```
/// n is the (hypothetical) number of bits to shift by.
/// f_n is a flag indicating whether this is really the n to shift by.
/// We match up bits[n..] with the sequence (2^0, 2^1, ...).
fn srl<P: PackedField>(bits: &[P; WORD_BITS], shift_amt: &[P; WORD_BITS]) -> P {
    shift_amt
        .iter()
        .enumerate()
        .flat_map(|(n, &f_n)| {
            bits.iter()
                .skip(n)
                .zip(P::Scalar::TWO.powers())
                .map(move |(&bit, base)| f_n * bit * base)
        })
        .sum()
}

/// Aithmetic shift towards the least significant bit.
///[ 0 0 0 1 0 0 0 1 ] input bits (LE)
///[ 0 0 0 1 0 0 0 0 ] shift_amt, indexed by n
///[ 1 0 0 0 1 1 1 1 ] output
///
/// n is the (hypothetical) number of bits to shift by.
/// f_n is a flag indicating whether this is really the n to shift by.
/// We repeat bits[WORD_SIZE - 1] n times, scaling by 2^(WORD_BITS - n)..2^WORD_BITS.
fn sra_ext<P: PackedField>(bits: &[P; WORD_BITS], shift_amt: &[P; WORD_BITS]) -> P {
    let ext_bit = *bits.last().unwrap();
    shift_amt
        .iter()
        .enumerate()
        .flat_map(|(n, &f_n)| {
            P::Scalar::TWO
                .powers()
                .skip(WORD_BITS - n)
                .take(n)
                .map(move |base| f_n * ext_bit * base)
        })
        .sum()
}

fn eval_all<P: PackedField>(lv: &LogicCols<P>, nv: &LogicCols<P>, cc: &mut ConstraintConsumer<P>) {
    let f_and = lv.op.f_and;
    let f_xor = lv.op.f_xor;
    let f_or = lv.op.f_or;
    let f_sll = lv.op.f_sll;
    let f_srl = lv.op.f_srl;
    let f_sra = lv.op.f_sra;

    // flags in {0, 1}
    cc.constraint(f_and * (f_and - P::ONES));
    cc.constraint(f_xor * (f_xor - P::ONES));
    cc.constraint(f_or * (f_or - P::ONES));
    cc.constraint(f_sll * (f_sll - P::ONES));
    cc.constraint(f_srl * (f_srl - P::ONES));
    cc.constraint(f_sra * (f_sra - P::ONES));

    // at most one op flag is set
    let flag_sum: P = f_and + f_xor + f_or;
    cc.constraint(flag_sum * (flag_sum - P::ONES));

    // input bit values in {0, 1}
    for bit in lv.in0 {
        cc.constraint(bit * (bit - P::ONES));
    }
    for bit in lv.in1 {
        cc.constraint(bit * (bit - P::ONES));
    }

    //TODO: constraints for AND, OR, XOR

    let sll_out = sll(&lv.in0, &lv.in1);
    cc.constraint(f_sll * (lv.out - sll_out));

    let srl_out = srl(&lv.in0, &lv.in1);
    cc.constraint(f_srl * (lv.out - srl_out));

    let sra_ext = sra_ext(&lv.in0, &lv.in1);
    let sra_out = srl_out + sra_ext;
    cc.constraint(f_sra * (lv.out - sra_out));
}

fn eval_all_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &LogicCols<ExtensionTarget<D>>,
    nv: &LogicCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LogicStark<F, const D: usize> {
    _unused: PhantomData<F>,
}

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for LogicStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, N_LOGIC_COLS, 0>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;

    type EvaluationFrameTarget =
        StarkFrame<ExtensionTarget<D>, ExtensionTarget<D>, N_LOGIC_COLS, 0>;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        frame: &Self::EvaluationFrame<FE, P, D2>,
        cc: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
        let local: &[P; N_LOGIC_COLS] = frame.get_local_values().try_into().unwrap();
        let local: &LogicCols<P> = local.borrow();
        let next: &[P; N_LOGIC_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &LogicCols<P> = next.borrow();
        eval_all(local, next, cc);
    }

    fn eval_ext_circuit(
        &self,
        cb: &mut CircuitBuilder<F, D>,
        frame: &Self::EvaluationFrameTarget,
        cc: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        let local: &[ExtensionTarget<D>; N_LOGIC_COLS] =
            frame.get_local_values().try_into().unwrap();
        let local: &LogicCols<ExtensionTarget<D>> = local.borrow();
        let next: &[ExtensionTarget<D>; N_LOGIC_COLS] = frame.get_next_values().try_into().unwrap();
        let next: &LogicCols<ExtensionTarget<D>> = next.borrow();
        eval_all_circuit(cb, local, next, cc);
    }

    fn constraint_degree(&self) -> usize {
        3
    }

    fn requires_ctls(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use plonky2::field::types::{Field, PrimeField64, Sample};
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use rand::Rng;
    use starky::stark_testing::{test_stark_circuit_constraints, test_stark_low_degree};

    use super::{sll, sra_ext, srl, LogicStark, WORD_BITS};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    type S = LogicStark<F, D>;

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

    fn to_le_bits<F: Field>(x: u32) -> [F; WORD_BITS] {
        core::array::from_fn(|i| F::from_bool(x & (1 << i) != 0))
    }

    #[test]
    fn test_sll() {
        let mut rng = rand::thread_rng();
        let x: u32 = rng.gen();
        let shift = rng.gen_range(0..WORD_BITS);
        let expect = F::from_canonical_u32(x << shift);

        let x_bits = to_le_bits(x);
        let mut shift_amt = [F::ZERO; WORD_BITS];
        shift_amt[shift] = F::ONE;
        let out = sll(&x_bits, &shift_amt);
        assert_eq!(out, expect);
    }

    #[test]
    fn test_srl() {
        let mut rng = rand::thread_rng();
        let x: u32 = rng.gen();
        let shift = rng.gen_range(0..WORD_BITS);
        let expect = F::from_canonical_u32(x >> shift);

        let x_bits = to_le_bits(x);
        let mut shift_amt = [F::ZERO; WORD_BITS];
        shift_amt[shift] = F::ONE;
        let out = srl(&x_bits, &shift_amt);
        assert_eq!(out, expect);
    }

    #[test]
    fn test_sra() {
        let mut rng = rand::thread_rng();
        let x: i32 = rng.gen();
        let shift = rng.gen_range(0..WORD_BITS);
        let expect = F::from_canonical_u32((x >> shift) as u32);

        let x_bits = to_le_bits(x as u32);
        let mut shift_amt = [F::ZERO; WORD_BITS];
        shift_amt[shift] = F::ONE;
        let srl_out = srl(&x_bits, &shift_amt);
        let sra_ext = sra_ext(&x_bits, &shift_amt);
        let out = srl_out + sra_ext;
        assert_eq!(out, expect);
    }
}
