pub mod columns;
pub mod stark;
pub mod trace;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Segment {
    Reg,
    Main,
}
