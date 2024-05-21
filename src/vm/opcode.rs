#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Opcode {
    ADD,
    SUB,
    SLT,
    SLTU,

    LW,
    SW,

    JAL,
    JALR,
}
