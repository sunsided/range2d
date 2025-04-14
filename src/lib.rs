//! # `range2d` — A 2D Range Iterator
//!
//! This crate provides [`Range2D`], a highly flexible, efficient, and composable iterator
//! for traversing a 2D rectangular coordinate space.
//!
//! It yields `(y, x)` coordinate pairs from a rectangular region defined by two `Range<usize>` bounds,
//! visiting each row in order from top to bottom and each column from left to right.
//!
//! ## Features
//!
//! - Forward and backward iteration (`DoubleEndedIterator`)
//! - Exact length tracking (`ExactSizeIterator`)
//! - Safe skipping with `.nth()`
//! - Efficient `split()` for parallel workloads
//! - `split_into(n)` for evenly sized chunks
//! - `chunks_of(n)` for fixed-size partitioning
//! - Resettable and reusable with `.reset()`
//!
//! ## Example
//!
//! ```rust
//! use range2d::Range2D;
//!
//! let iter = Range2D::new(0..2, 0..3);
//! let coords: Vec<_> = iter.collect();
//!
//! assert_eq!(coords, vec![
//!     (0, 0), (0, 1), (0, 2),
//!     (1, 0), (1, 1), (1, 2),
//! ]);
//! ```
//!
//! ## Use Cases
//!
//! - Tile maps
//! - Grid-based simulations
//! - Procedural generation
//! - Image or framebuffer traversal
//!
//! ## Integration
//!
//! This iterator is compatible with all iterator adapters (`.rev()`, `.take()`, `.map()`, etc.),
//! and behaves predictably when fused or split into subranges.

use std::iter::FusedIterator;
use std::ops::Range;

/// A 2D coordinate iterator over `(y, x)` pairs.
#[derive(Debug, Clone)]
pub struct Range2D {
    y_range: Range<usize>,
    x_range: Range<usize>,
    start: usize,
    end: usize,
}

impl Range2D {
    /// Creates a new iterator over the given coordinate rectangle.
    pub fn new(y_range: Range<usize>, x_range: Range<usize>) -> Self {
        let height = y_range.end.saturating_sub(y_range.start);
        let width = x_range.end.saturating_sub(x_range.start);
        let total = (height * width) as usize;

        Self {
            y_range,
            x_range,
            start: 0,
            end: total,
        }
    }

    pub fn full(height: usize, width: usize) -> Self {
        Self::new(0..height, 0..width)
    }

    /// Resets the iterator to its full original range.
    pub fn reset(&mut self) {
        self.start = 0;
        self.end = self.total_len();
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of remaining `(y, x)` coordinate pairs that will be yielded by this iterator.
    ///
    /// This takes into account how far iteration has already progressed.
    ///
    /// # Example
    /// ```
    /// # use range2d::Range2D;
    /// let mut iter = Range2D::new(2..4, 5..7); // yields 4 items total
    /// assert_eq!(iter.len(), 4);
    ///
    /// iter.next(); // (2, 5)
    /// assert_eq!(iter.len(), 3);
    ///
    /// iter.for_each(drop);
    /// assert_eq!(iter.len(), 0);
    /// ```
    ///
    /// This is **not** equivalent to `y_range.len() * x_range.len()` unless the iterator is at the start.
    /// Instead, it computes the number of rows remaining (`y_range.end - cur_y`) and adjusts for
    /// how far we are into the current row (`cur_x`).
    ///
    /// See also [`ExactSizeIterator`].
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns the total size of the full 2D range, without regard to progress.
    pub fn total_len(&self) -> usize {
        let height = self.y_range.end.saturating_sub(self.y_range.start);
        let width = self.x_range.end.saturating_sub(self.x_range.start);
        (height * width) as usize
    }

    /// Returns two disjoint iterators that cover the remaining range.
    /// After calling this, the original iterator is no longer usable.
    ///
    /// # Example
    /// ```
    /// use range2d::Range2D;
    /// let iter = Range2D::new(0..2, 0..4); // 8 items
    /// let (left, right) = iter.split();
    ///
    /// assert_eq!(left.count(), 4);
    /// assert_eq!(right.count(), 4);
    /// ```
    pub fn split(&self) -> (Self, Self) {
        let mid = self.start + self.len() / 2;

        let left = Self {
            start: self.start,
            end: mid,
            ..self.clone()
        };

        let right = Self {
            start: mid,
            end: self.end,
            ..self.clone()
        };

        (left, right)
    }

    /// Splits the remaining range into `n` disjoint chunks of approximately equal size.
    /// The resulting iterators can be used in parallel or sequentially.
    pub fn split_into(&self, n: usize) -> Vec<Self> {
        if n == 0 {
            return Vec::new();
        }

        let total = self.len();
        let base = total / n;
        let rem = total % n;

        let mut result = Vec::with_capacity(n);
        let mut current_start = self.start;

        for i in 0..n {
            let chunk_len = base + if i < rem { 1 } else { 0 };
            let chunk_end = current_start + chunk_len;

            result.push(Self {
                start: current_start,
                end: chunk_end,
                ..self.clone()
            });

            current_start = chunk_end;
        }

        result
    }

    /// Splits the remaining range into fixed-size chunks of at most `chunk_size` items each.
    /// The last chunk may be smaller.
    pub fn chunks_of(&self, chunk_size: usize) -> Vec<Self> {
        if chunk_size == 0 {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut current_start = self.start;

        while current_start < self.end {
            let chunk_end = (current_start + chunk_size).min(self.end);

            result.push(Self {
                start: current_start,
                end: chunk_end,
                ..self.clone()
            });

            current_start = chunk_end;
        }

        result
    }

    const fn index_to_coord(&self, index: usize) -> (usize, usize) {
        let width = self.x_range.end - self.x_range.start;
        if width == 0 {
            return (0, 0);
        }

        let y = self.y_range.start + (index as usize / width);
        let x = self.x_range.start + (index as usize % width);
        (y, x)
    }
}

impl Iterator for Range2D {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let coord = self.index_to_coord(self.start);
        self.start += 1;
        Some(coord)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.start = self.start.saturating_add(n);
        self.next()
    }
}

impl DoubleEndedIterator for Range2D {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        self.end -= 1;
        Some(self.index_to_coord(self.end))
    }
}

impl ExactSizeIterator for Range2D {}
impl FusedIterator for Range2D {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_tile_chunk_bounds() {
        let h = 3;
        let w = 4;
        let coords: Vec<(usize, usize)> = Range2D::full(h, w).collect();

        let expected: Vec<(usize, usize)> =
            (0..h).flat_map(|y| (0..w).map(move |x| (y, x))).collect();

        assert_eq!(coords, expected);
    }

    #[test]
    fn test_custom_ranges() {
        let coords: Vec<_> = Range2D::new(2..4, 5..7).collect();
        let expected = vec![(2, 5), (2, 6), (3, 5), (3, 6)];
        assert_eq!(coords, expected);
    }

    #[test]
    fn test_empty_x_range() {
        let coords: Vec<_> = Range2D::new(0..5, 3..3).collect();
        assert!(coords.is_empty());
    }

    #[test]
    fn test_empty_y_range() {
        let coords: Vec<_> = Range2D::new(2..2, 0..10).collect();
        assert!(coords.is_empty());
    }

    #[test]
    fn test_single_element() {
        let coords: Vec<_> = Range2D::new(3..4, 7..8).collect();
        assert_eq!(coords, vec![(3, 7)]);
    }

    #[test]
    fn test_iter_next_behavior() {
        let mut iter = Range2D::new(1..3, 0..2).into_iter();
        assert_eq!(iter.next(), Some((1, 0)));
        assert_eq!(iter.next(), Some((1, 1)));
        assert_eq!(iter.next(), Some((2, 0)));
        assert_eq!(iter.next(), Some((2, 1)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_next_back_only() {
        let iter = Range2D::new(0..2, 0..3); // 6 tiles
        let coords: Vec<_> = iter.rev().collect();

        let expected = vec![(1, 2), (1, 1), (1, 0), (0, 2), (0, 1), (0, 0)];
        assert_eq!(coords, expected);
    }

    #[test]
    fn test_mixed_next_and_next_back() {
        let mut iter = Range2D::new(0..2, 0..3); // 6 tiles

        assert_eq!(iter.next(), Some((0, 0))); // forward
        assert_eq!(iter.next_back(), Some((1, 2))); // backward
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next_back(), Some((1, 1)));
        assert_eq!(iter.next(), Some((0, 2)));
        assert_eq!(iter.next_back(), Some((1, 0)));
        assert_eq!(iter.next(), None); // exhausted
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_double_ended_len_and_exhaustion() {
        let mut iter = Range2D::new(5..6, 10..14); // 4 tiles: (5,10..14)

        assert_eq!(iter.len(), 4);
        iter.next(); // (5,10)
        assert_eq!(iter.len(), 3);
        iter.next_back(); // (5,13)
        assert_eq!(iter.len(), 2);
        iter.next(); // (5,11)
        iter.next(); // (5,12)
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_split_preserves_order() {
        let range = Range2D::new(0..2, 0..4); // 8 tiles
        let (left, right) = range.split();

        let all: Vec<_> = left.chain(right).collect();

        let expected = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0),
            (1, 1),
            (1, 2),
            (1, 3),
        ];
        assert_eq!(all, expected);
    }

    #[test]
    fn test_split_into_chunks() {
        let iter = Range2D::new(0..2, 0..4); // 8 tiles
        let chunks = iter.split_into(3);
        let sizes: Vec<_> = chunks.iter().map(|c| c.len()).collect();
        assert_eq!(sizes, vec![3, 3, 2]);

        let all: Vec<_> = chunks.into_iter().flat_map(|c| c).collect();
        let expected = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0),
            (1, 1),
            (1, 2),
            (1, 3),
        ];
        assert_eq!(all, expected);
    }

    #[test]
    fn test_nth() {
        let mut iter = Range2D::new(1..3, 4..8); // 8 tiles
        assert_eq!(iter.nth(0), Some((1, 4)));
        assert_eq!(iter.nth(2), Some((1, 7))); // skip two, get (1,7)
        assert_eq!(iter.nth(10), None); // too far, exhausted
    }
}
