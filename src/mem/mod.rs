pub mod columns;
pub mod stark;

const SEG_SCALE_FACTOR: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Segment {
    Reg = 0,
    Main = 1 << SEG_SCALE_FACTOR,
}

impl Segment {
    pub fn unscale(&self) -> usize {
        *self as usize >> SEG_SCALE_FACTOR
    }
}
