#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Opcode {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Sltu,
    Lw,
    Sw,
    Jal,
    Jalr,
    Beq,
    Bne,
    Bltu,
    Bgeu,
}
