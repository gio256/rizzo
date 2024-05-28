use crate::iter::Zip;

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
    fn zip<U>(self, other: U) -> Zip<Self, U::IntoIter>
    where
        Self: Sized,
        U: IntoIterator,
    {
        Zip::new(self, other.into_iter())
    }
}
