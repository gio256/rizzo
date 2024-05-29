use crate::iter::{Lend, Zip};

/// A lending iterator trait that uses a [generic associated type][gat] to
/// allow the [`next`] method to return an item that borrows from `self`.
///
/// In particular, this enables the use of [`windows_mut`] to access
/// overlapping mutable windows on a slice. This is a minimal implementation
/// with only the methods needed in this crate.
///
/// [gat]: https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
/// [`next`]: Self::next
/// [`windows_mut`]: crate::iter::windows_mut
pub trait LendIter {
    /// The type of the elements being iterated over.
    type Item<'n>
    where
        Self: 'n;

    /// Advances the iterator and returns the next value.
    ///
    /// Returns `None` when iteration is finished. Note that this cannot be
    /// used with Rust's [`for`] loop syntactic sugar.
    ///
    /// [`for`]: https://doc.rust-lang.org/std/keyword.for.html
    fn next(&mut self) -> Option<Self::Item<'_>>;

    /// 'Zips up' an implementer of [`LendIter`] with a standard [`Iterator`].
    ///
    /// [`LendIter`]: Self
    /// [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
    fn zip_iter<U>(self, other: U) -> Zip<Self, Lend<U::IntoIter>>
    where
        Self: Sized,
        U: IntoIterator,
    {
        Zip::new(self, Lend::from_iter(other))
    }
}

// pub trait IntoLendIter {
//     type Item<'n> where Self: 'n;
//     type IntoLend<'n>: LendIter<Item<'n> = Self::Item<'n>> where Self: 'n;
//     fn into_lend<'a>(self) -> Self::IntoLend<'a>;
// }
// impl<I: LendIter> IntoLendIter for I {
//     type Item<'n> = I::Item<'n> where I: 'n;
//     type IntoLend<'n> = I where I: 'n;
//     #[inline]
//     fn into_lend<'a>(self) -> I where I: 'a {
//         self
//     }
// }
