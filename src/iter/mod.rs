pub trait LendIter {
    type Item<'n>
    where
        Self: 'n;

    fn next(&mut self) -> Option<Self::Item<'_>>;

    fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter>
    where
        Self: Sized,
        U: IntoIterator,
    {
        Zip::new(self, other.into_iter())
    }
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Zip<A, B> {
    a: A,
    b: B,
}

impl<A, B> Zip<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: LendIter, B: Iterator> LendIter for Zip<A, B> {
    type Item<'n> = (A::Item<'n>, B::Item) where Self: 'n;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let a = self.a.next()?;
        let b = self.b.next()?;
        Some((a, b))
    }
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct WindowsMut<T, const SIZE: usize> {
    v: T,
    idx: usize,
}

impl<T, const SIZE: usize> WindowsMut<T, SIZE> {
    pub fn new(v: T) -> Self {
        Self { v, idx: 0 }
    }
}

impl<'s, T, const SIZE: usize> LendIter for WindowsMut<&'s mut [T], SIZE> {
    type Item<'n> = &'n mut [T; SIZE] where Self: 'n;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let res = self.v.get_mut(self.idx..)?.get_mut(..SIZE)?;
        self.idx += 1;
        Some(res.try_into().unwrap())
    }
}

pub fn windows_mut<T, const SIZE: usize>(v: &mut [T]) -> WindowsMut<&mut [T], SIZE> {
    WindowsMut::new(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_zip() {
        let mut xs = [0, 1, 2, 3, 4, 5];
        let ys = [6, 7, 8, 9];

        let windows = windows_mut::<_, 2>(&mut xs);
        let mut iter = windows.zip(ys);
        let mut expect = [(0, 1, 6), (1, 2, 7), (2, 3, 8), (3, 4, 9)].into_iter();
        while let Some(([a, b], c)) = iter.next() {
            assert_eq!((*a, *b, c), expect.next().unwrap());
        }
    }
}
