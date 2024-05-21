#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Opcode {
    ADD,
    SUB,
    SLTU,

    LW,
    SW,

    JAL,
    JALR,

    BEQ,
    BNE,
    BLTU,
    BGEU,
}
