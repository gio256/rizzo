/// Note that these opcodes are not one-to-one with riscv instructions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum Opcode {
    // arithmetic
    ADD,
    SUB,
    SLT,
    SLTU,

    // logic
    AND,
    OR,
    XOR,

    // memory load ops
    LW,
    LB,
    LH,
    LBU,
    LHU,

    // memory store ops
    SW,
    SB,
    SH,

    // jumps
    JAL,
    JALR,

    // branching
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
}
