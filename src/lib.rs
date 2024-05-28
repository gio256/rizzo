#![feature(array_windows)]
#![allow(unused)]

pub mod arith;
pub mod cpu;
pub mod iter;
pub mod mem;
pub mod pack;
pub mod stark;
pub mod util;
pub mod vm;

mod ctl_test;

#[cfg(test)]
mod tests {
    #[test]
    fn test_true() {
        assert!(true);
    }
}
