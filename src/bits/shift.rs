use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::util::felt_from_le_bits;
use crate::bits::columns::{BitCols, WORD_BITS};

/// Logical shift towards the most significant bit.
fn sll<P: PackedField>(bits: &[P; WORD_BITS], shift_amt: &[P; WORD_BITS]) -> P {
    // [ 1 0 0 0 1 0 0 0 ] input bits (LE).
    // [ 0 0 0 1 0 0 0 0 ] shift_amt, indexed by n.
    // [ 0 0 0 1 0 0 0 1 ] output bits (LE).
    //
    // n is the (hypothetical) number of bits to shift by.
    // f_n is a flag indicating whether this is really the n to shift by.
    // All bits in the output with index < n are 0.
    // We match up bits[..WORD_BITS - n] with the sequence (2^n, 2^n+1, ...).
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
fn srl<P: PackedField>(bits: &[P; WORD_BITS], shift_amt: &[P; WORD_BITS]) -> P {
    // [ 0 0 0 1 0 0 0 1 ] input bits (LE).
    // [ 0 0 0 1 0 0 0 0 ] shift_amt, indexed by n.
    // [ 1 0 0 0 1 0 0 0 ] output bits (LE).
    //
    // n is the (hypothetical) number of bits to shift by.
    // f_n is a flag indicating whether this is really the n to shift by.
    // We match up bits[n..] with the sequence (2^0, 2^1, ...).
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

/// Aithmetic shift towards the least significant bit. Returns a field element
/// representing the value of the sign extension bits only. This should be added
/// to the output of [`srl`] to get the expected output of the `SRA` instruction.
fn sra_ext<P: PackedField>(bits: &[P; WORD_BITS], shift_amt: &[P; WORD_BITS]) -> P {
    // [ 0 0 0 1 0 0 0 1 ] input bits (LE).
    // [ 0 0 0 1 0 0 0 0 ] shift_amt, indexed by n.
    // [ 1 0 0 0 1 1 1 1 ] output bits (LE).
    //
    // n is the (hypothetical) number of bits to shift by.
    // f_n is a flag indicating whether this is really the n to shift by.
    // We repeat bits[WORD_SIZE - 1] n times, scaling by 2^(WORD_BITS - n)..2^WORD_BITS.
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

/// Constraints for AND, OR, and XOR from
/// [zk_evm](https://github.com/0xPolygonZero/zk_evm/blob/677dc0dc066d15209773ce1e7c990df8a845da98/evm_arithmetization/src/logic.rs#L310).
pub(crate) fn eval<P: PackedField>(lv: &BitCols<P>, cc: &mut ConstraintConsumer<P>) {
    let f_sll = lv.op.f_sll;
    let f_srl = lv.op.f_srl;
    let f_sra = lv.op.f_sra;
    let out = lv.out;

    // SLL
    let sll_out = sll(&lv.in0, &lv.in1);
    cc.constraint(f_sll * (out - sll_out));

    // SRL
    let srl_out = srl(&lv.in0, &lv.in1);
    cc.constraint(f_srl * (out - srl_out));

    // SRA
    let sra_ext = sra_ext(&lv.in0, &lv.in1);
    let sra_out = srl_out + sra_ext;
    cc.constraint(f_sra * (out - sra_out));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &BitCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
}

#[cfg(test)]
mod tests {
    use plonky2::field::types::{Field, PrimeField64, Sample};
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use rand::Rng;

    use super::*;
    use crate::util::u32_to_le_bits;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_sll() {
        let mut rng = rand::thread_rng();
        let x: u32 = rng.gen();
        let shift = rng.gen_range(0..WORD_BITS);
        let expect = F::from_canonical_u32(x << shift);

        let x_bits = u32_to_le_bits(x);
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

        let x_bits = u32_to_le_bits(x);
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

        let x_bits = u32_to_le_bits(x as u32);
        let mut shift_amt = [F::ZERO; WORD_BITS];
        shift_amt[shift] = F::ONE;
        let srl_out = srl(&x_bits, &shift_amt);
        let sra_ext = sra_ext(&x_bits, &shift_amt);
        let out = srl_out + sra_ext;
        assert_eq!(out, expect);
    }
}
