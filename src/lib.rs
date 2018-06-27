//! [`Zom`] is a generic collection for either zero, one or many elements. It
//! is intended for situations where you usually expect only zero or one items
//! and does not require allocation in these cases.
//!
//! [`Zom`]: enum.Zom.html

#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::{mem, slice, vec};

/// A collection of zero, one or many elements.
#[derive(Debug)]
pub enum Zom<T> {
    Zero,
    One(T),
    Many(Vec<T>),
}

impl<T> Zom<T> {
    /// Adds a new element to the collection.
    pub fn push(&mut self, val: T) {
        match *self {
            Zom::Zero => *self = Zom::One(val),
            ref mut this => this.to_vec().push(val),
        }
    }

    /// Removes the last element from the collection. When called on a
    /// `Zom::Many` with one element it remain in the `Zom::Many` state.
    pub fn pop(&mut self) -> Option<T> {
        match self.take() {
            Zom::Zero => None,
            Zom::One(one) => Some(one),
            Zom::Many(mut many) => {
                let val = many.pop();
                *self = Zom::Many(many);
                val
            }
        }
    }

    /// Converts the `Zom` to the `Zom::Many` variant and returns a mutable
    /// references to it.
    pub fn to_vec(&mut self) -> &mut Vec<T> {
        let vec = match self.take() {
            Zom::Zero => vec![],
            Zom::One(one) => vec![one],
            Zom::Many(many) => many,
        };
        *self = Zom::Many(vec);
        match self {
            Zom::Many(many) => many,
            _ => unreachable!(),
        }
    }

    /// Removes all elements from the `Zom`, without deallocating any memory.
    pub fn clear(&mut self) {
        match self {
            Zom::Many(many) => many.clear(),
            this @ Zom::One(_) => *this = Zom::Zero,
            Zom::Zero => (),
        }
    }

    /// Minimizes the memory allocated by the `Zom`. If it is `Zom::Many` but
    /// only contains zero or one elements, it is converted to the appropriate
    /// variant.
    pub fn shrink_to_fit(&mut self) {
        let this = match self.take() {
            Zom::Many(mut many) => match many.len() {
                0 => Zom::Zero,
                1 => Zom::One(many.into_iter().next().unwrap()),
                _ => {
                    many.shrink_to_fit();
                    Zom::Many(many)
                }
            },
            this => this,
        };
        *self = this;
    }

    /// Replace the contents of the `Zom` with `Zom::Zero`, returning the old value.
    pub fn take(&mut self) -> Zom<T> {
        mem::replace(self, Zom::Zero)
    }

    /// Returns an iterator over the `Zom`.
    pub fn iter(&self) -> slice::Iter<T> {
        self.deref().iter()
    }

    /// Returns a mutable iterator over the `Zom`.
    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        self.deref_mut().iter_mut()
    }

    /// Returns consuming iterator over the `Zom`.
    pub fn into_iter(self) -> IntoIter<T> {
        let inner = match self {
            Zom::Zero => IntoIterInner::Zero,
            Zom::One(one) => IntoIterInner::One(one),
            Zom::Many(many) => IntoIterInner::Many(many.into_iter()),
        };

        IntoIter { inner }
    }
}

impl<T: Clone> Clone for Zom<T> {
    fn clone(&self) -> Self {
        match *self {
            Zom::Zero => Zom::Zero,
            Zom::One(ref one) => Zom::One(one.clone()),
            Zom::Many(ref many) => match many.len() {
                0 => Zom::Zero,
                1 => Zom::One(many[0].clone()),
                _ => Zom::Many(many.clone()),
            },
        }
    }

    fn clone_from(&mut self, src: &Self) {
        match (self, src) {
            (&mut Zom::Many(ref mut lhs), &Zom::Many(ref rhs))
                if lhs.capacity() >= rhs.len() || rhs.len() > 1 =>
            {
                lhs.clone_from(rhs)
            }
            (&mut ref mut lhs, &Zom::Many(ref rhs)) => match rhs.len() {
                0 => *lhs = Zom::Zero,
                1 => *lhs = Zom::One(rhs[0].clone()),
                _ => *lhs = Zom::Many(rhs.clone()),
            },
            (lhs, rhs) => *lhs = rhs.clone(),
        }
    }
}

impl<T> Deref for Zom<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        match *self {
            Zom::Zero => &[],
            Zom::One(ref one) => slice_from_ref(one),
            Zom::Many(ref many) => many,
        }
    }
}

impl<T> DerefMut for Zom<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        match *self {
            Zom::Zero => &mut [],
            Zom::One(ref mut one) => slice_from_mut(one),
            Zom::Many(ref mut many) => many,
        }
    }
}

impl<T: Eq> Eq for Zom<T> {}

impl<T: Ord> Ord for Zom<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(self.deref(), other.deref())
    }
}

impl<T: PartialEq> PartialEq for Zom<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.deref(), other.deref())
    }
}

impl<T: PartialOrd> PartialOrd for Zom<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.deref(), other.deref())
    }
}

impl<T> Default for Zom<T> {
    #[inline]
    fn default() -> Self {
        Zom::Zero
    }
}

impl<T: Hash> Hash for Zom<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T> AsRef<[T]> for Zom<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<T> AsMut<[T]> for Zom<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

/// The result of calling [`Zom::into_iter`].
///
/// [`Zom::into_iter`]: enum.IntoIter.html#method.into_iter
#[derive(Debug)]
pub struct IntoIter<T> {
    inner: IntoIterInner<T>,
}

#[derive(Debug)]
enum IntoIterInner<T> {
    Zero,
    One(T),
    Many(vec::IntoIter<T>),
}

impl<T> IntoIter<T> {
    /// Returns the remaining items of this iterator as a slice.
    pub fn as_slice(&self) -> &[T] {
        match self.inner {
            IntoIterInner::Zero => &[],
            IntoIterInner::One(ref one) => slice_from_ref(one),
            IntoIterInner::Many(ref many) => many.as_slice(),
        }
    }

    /// Returns the remaining items of this iterator as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self.inner {
            IntoIterInner::Zero => &mut [],
            IntoIterInner::One(ref mut one) => slice_from_mut(one),
            IntoIterInner::Many(ref mut many) => many.as_mut_slice(),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match mem::replace(&mut self.inner, IntoIterInner::Zero) {
            IntoIterInner::Zero => None,
            IntoIterInner::One(one) => Some(one),
            IntoIterInner::Many(mut many) => {
                let next = many.next();
                self.inner = IntoIterInner::Many(many);
                next
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.as_slice().len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.as_slice().len()
    }
}

impl<T> IntoIterator for Zom<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Zom<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Zom<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> FromIterator<T> for Zom<T> {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        let first = match iter.next() {
            None => return Zom::Zero,
            Some(val) => val,
        };
        let second = match iter.next() {
            None => return Zom::One(first),
            Some(val) => val,
        };
        let mut many = Vec::with_capacity(iter.size_hint().0 + 2);
        many.push(first);
        many.push(second);
        many.extend(iter);
        Zom::Many(many)
    }
}

impl<T> Extend<T> for Zom<T> {
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        if let Some(first) = iter.next() {
            let this = match self.take() {
                Zom::Zero => Zom::One(first),
                Zom::One(one) => {
                    let mut many = Vec::with_capacity(iter.size_hint().0 + 2);
                    many.push(one);
                    many.push(first);
                    many.extend(iter);
                    Zom::Many(many)
                }
                Zom::Many(mut many) => {
                    many.reserve(iter.size_hint().0 + 2);
                    many.push(first);
                    many.extend(iter);
                    Zom::Many(many)
                }
            };
            *self = this;
        }
    }
}

impl<T> From<T> for Zom<T> {
    #[inline]
    fn from(one: T) -> Self {
        Zom::One(one)
    }
}

impl<T> From<Vec<T>> for Zom<T> {
    #[inline]
    fn from(many: Vec<T>) -> Self {
        Zom::Many(many)
    }
}

// TODO: replace this with slice::from_ref when it is stable.
fn slice_from_ref<T>(s: &T) -> &[T] {
    unsafe { slice::from_raw_parts(s, 1) }
}

// TODO: replace this with slice::from_mut when it is stable.
fn slice_from_mut<T>(s: &mut T) -> &mut [T] {
    unsafe { slice::from_raw_parts_mut(s, 1) }
}
