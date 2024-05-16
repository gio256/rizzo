pub(crate) const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = i;
        i += 1;
    }
    arr
}

// https://github.com/0xPolygonZero/zk_evm
pub(crate) unsafe fn transmute_no_compile_time_size_checks<T, U>(value: T) -> U {
    debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<U>());
    // Need ManuallyDrop so that `value` is not dropped by this function.
    let value = core::mem::ManuallyDrop::new(value);
    // Copy the bit pattern. The original value is no longer safe to use.
    core::mem::transmute_copy(&value)
}
