use core::hint::assert_unchecked;
use core::ops::{Index, IndexMut};
use ufmt::{uDebug, uWrite, Formatter};

/// An array with a fixed in-memory size. The array dynamically stores the number of items currently
/// in use within the array.
///
/// Pushing new items can fail if there is no room.
///
/// The array is always initialized.
///
/// Note Borrow and BorrowMut are not currently implemented because if they were, we'd need to also
/// manually implement PartialOrd, Ord and Eq to make sure these methods all work with respect to
/// the slice.
#[derive(Copy, Clone)]
pub struct FixedArr<T, const N: usize> {
    // SAFETY INVARIANT: len < n.
    len: usize,
    items: [T; N],
}

impl<T, const N: usize> FixedArr<T, N> {
    /// Make a new empty FixedArr. This variant is the most efficient, but it requires that the
    /// type implement Copy.
    pub fn new() -> Self where T: Default + Copy {
        Self {
            len: 0,
            items: [T::default(); N]
        }
    }

    pub fn new_from_example(def: T) -> Self where T: Clone {
        Self {
            len: 0,
            items: core::array::repeat(def)
        }
    }

    /// When T does not impl Default or Copy, this
    pub fn new_from_fn(f: impl FnMut(usize) -> T) -> Self {
        Self {
            len: 0,
            items: core::array::from_fn(f)
        }
    }

    pub fn len(&self) -> usize { self.len }

    /// Try to push the specified item into the array. This will fail if there is not space for the
    /// item. In this case, the item is returned to the caller via the Result::Err variant.
    pub fn try_push(&mut self, item: T) -> Result<(), T> {
        if self.len < N {
            self.items[self.len] = item;
            self.len += 1;
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Push the item to the end of the array. This method panics if the array does not have space.
    pub fn push(&mut self, item: T) {
        assert!(self.len < N, "Fixed size array exhausted");
        self.items[self.len] = item;
        self.len += 1;
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: self.len is always <= N, so this is valid.
        unsafe { assert_unchecked(self.len <= N); }
        &self.items[0..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: self.len is always <= N, so this is valid.
        unsafe { assert_unchecked(self.len <= N); }
        &mut self.items[0..self.len]
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }
}

impl<T, const N: usize> Default for FixedArr<T, N> where T: Default + Copy {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a FixedArr<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T, const N: usize> AsRef<[T]> for FixedArr<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> AsMut<[T]> for FixedArr<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> uDebug for FixedArr<T, N> where T: uDebug {
    fn fmt<W>(&self, fmt: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized
    {
        fmt.debug_list()?
            .entries(self.as_slice())?
            .finish()
    }
}

impl<T, const N: usize> Index<usize> for FixedArr<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "Index out of bounds");

        &self.items[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for FixedArr<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "Index out of bounds");

        &mut self.items[index]
    }
}