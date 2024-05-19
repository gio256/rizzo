#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Opcode {
    ADD,
    SUB,
    SLT,

    LW,
    SW,

    JAL,
    JALR,
}
