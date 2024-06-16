use itertools::izip;
use plonky2::field::extension::Extendable;
use plonky2::field::packed::PackedField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use starky::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};

use crate::util::felt_from_le_bits;
use crate::logic::columns::{LogicCols, LOGIC_COL_MAP};

/// Constraints for AND, OR, and XOR from
/// [zk_evm](https://github.com/0xPolygonZero/zk_evm/blob/677dc0dc066d15209773ce1e7c990df8a845da98/evm_arithmetization/src/logic.rs#L310).
pub(crate) fn eval<P: PackedField>(lv: &LogicCols<P>, cc: &mut ConstraintConsumer<P>) {
    let f_and = lv.op.f_and;
    let f_xor = lv.op.f_xor;
    let f_or = lv.op.f_or;

    // `x op y = sum_coeff * (x + y) + and_coeff * (x & y)` where
    // `AND => sum_coeff = 0, and_coeff = 1`
    // `OR  => sum_coeff = 1, and_coeff = -1`
    // `XOR => sum_coeff = 1, and_coeff = -2`
    let sum_coeff = f_or + f_xor;
    let and_coeff = f_and - f_or - f_xor * P::Scalar::TWO;
    let f_logic = f_and + f_xor + f_or;

    // `in0 & in1` reconstructed as a single field element.
    let x_and_y: P = izip!(lv.in0, lv.in1, P::Scalar::TWO.powers())
        .map(|(x_bit, y_bit, base)| x_bit * y_bit * base)
        .sum();

    // Ensure `lv.and` contains the correct result, used to lower the degree
    // of the output constraint.
    cc.constraint(f_logic * (lv.and - x_and_y));

    // in0 and in1 reconstructed as field elements.
    let x = felt_from_le_bits(lv.in0);
    let y = felt_from_le_bits(lv.in1);

    // Output constraint for AND, OR, and XOR.
    let x_op_y = sum_coeff * (x + y) + and_coeff * lv.and;
    cc.constraint(f_logic * (lv.out - x_op_y));
}

pub(crate) fn eval_circuit<F: RichField + Extendable<D>, const D: usize>(
    cb: &mut CircuitBuilder<F, D>,
    lv: &LogicCols<ExtensionTarget<D>>,
    cc: &mut RecursiveConstraintConsumer<F, D>,
) {
    //TODO
}
