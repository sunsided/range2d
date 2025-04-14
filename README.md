# `range2d` — A 2D Range Iterator

This crate provides `Range2D`, a highly flexible, efficient, and composable iterator
for traversing a 2D rectangular coordinate space.

It yields `(y, x)` coordinate pairs from a rectangular region defined by two `Range<usize>` bounds,
visiting each row in order from top to bottom and each column from left to right.

## Features

- Forward and backward iteration (`DoubleEndedIterator`)
- Exact length tracking (`ExactSizeIterator`)
- Safe skipping with `.nth()`
- Efficient `split()` for parallel workloads
- `split_into(n)` for evenly sized chunks
- `chunks_of(n)` for fixed-size partitioning
- Resettable and reusable with `.reset()`

## Example

```rust
use range2d::Range2D;

fn example() {
    let iter = Range2D::new(0..2, 0..3);
    let coords: Vec<_> = iter.collect();

    assert_eq!(coords, vec![
        (0, 0), (0, 1), (0, 2),
        (1, 0), (1, 1), (1, 2),
    ]);
}
```

## Use Cases

- Tile maps
- Grid-based simulations
- Procedural generation
- Image or framebuffer traversal

## Integration

This iterator is compatible with all iterator adapters (`.rev()`, `.take()`, `.map()`, etc.),
and behaves predictably when fused or split into subranges.
