/// Returns [0,1,..,N]
pub(crate) const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = i;
        i += 1;
    }
    arr
}

pub(crate) fn fst<A, B>(x: (A, B)) -> A {
    x.0
}

pub(crate) fn snd<A, B>(x: (A, B)) -> B {
    x.1
}
