pub trait Idx:
    Copy
    + PartialOrd
    + Eq
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Rem<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn from_usize(n: usize) -> Self;
    fn to_usize(self) -> usize;
}

macro_rules! impl_idx_for {
    ($($t:ty),*) => {
        $(
            impl Idx for $t {
                fn zero() -> Self { 0 }
                fn one() -> Self { 1 }
                fn saturating_sub(self, rhs: Self) -> Self {
                    self.saturating_sub(rhs)
                }
                fn from_usize(n: usize) -> Self {
                    n as $t
                }
                fn to_usize(self) -> usize {
                    self as usize
                }
            }
        )*
    };
}

macro_rules! impl_idx_smoke_tests {
    ($($name:ident: $t:ty),* $(,)?) => {
        $(
        #[cfg(test)]
        mod $name {
            use super::*;
            use $crate::*;

            #[test]
            fn test_trait() {
                let z: $t = <$t>::zero();
                let o: $t = <$t>::one();
                assert_eq!(z, 0 as $t);
                assert_eq!(o, 1 as $t);
                assert_eq!(<$t>::to_usize(42 as $t), 42);
                assert_eq!(<$t>::from_usize(42), 42 as $t);
                let a: $t = <$t>::from_usize(7);
                let b: $t = <$t>::from_usize(5);
                // For saturating_sub, the expectation is that if a < b, it saturates to 0.
                assert_eq!(a.saturating_sub(b), 2 as $t);
            }

            #[test]
            fn range2d_iterator_collect() {
                // Construct a Range2D with 2 rows, 3 columns
                let iter = Range2D::<$t>::new(
                    <$t>::zero()..<$t>::from_usize(2),
                    <$t>::zero()..<$t>::from_usize(3)
                );
                let collected: Vec<($t, $t)> = iter.collect();
                let expected: Vec<($t, $t)> = vec![
                    (<$t>::zero(), <$t>::zero()),
                    (<$t>::zero(), <$t>::one()),
                    (<$t>::zero(), <$t>::from_usize(2)),
                    (<$t>::one(),  <$t>::zero()),
                    (<$t>::one(),  <$t>::one()),
                    (<$t>::one(),  <$t>::from_usize(2)),
                ];
                assert_eq!(collected, expected, "Collect test failed for type {}", stringify!($t));
            }

            #[test]
            fn range2d_iterator_rev() {
                let iter = Range2D::<$t>::new(
                    <$t>::zero()..<$t>::from_usize(2),
                    <$t>::zero()..<$t>::from_usize(3)
                );
                let collected: Vec<($t, $t)> = iter.rev().collect();
                let expected: Vec<($t, $t)> = vec![
                    (<$t>::one(),  <$t>::from_usize(2)),
                    (<$t>::one(),  <$t>::one()),
                    (<$t>::one(),  <$t>::zero()),
                    (<$t>::zero(), <$t>::from_usize(2)),
                    (<$t>::zero(), <$t>::one()),
                    (<$t>::zero(), <$t>::zero()),
                ];
                assert_eq!(collected, expected, "Reverse test failed for type {}", stringify!($t));
            }

            #[test]
            fn range2d_iterator_nth() {
                let mut iter = Range2D::<$t>::new(
                    <$t>::zero()..<$t>::from_usize(2),
                    <$t>::zero()..<$t>::from_usize(3)
                );
                // nth(1) skips the first element and returns the second.
                let nth_elem = iter.nth(1);
                // Expected sequence:
                // index 0: (0, 0)
                // index 1: (0, 1)
                assert_eq!(nth_elem, Some((<$t>::zero(), <$t>::one())),
                    "nth test failed for type {}", stringify!($t));
            }

            #[test]
            fn range2d_iterator_size_hint() {
                let iter = Range2D::<$t>::new(
                    <$t>::zero()..<$t>::from_usize(2),
                    <$t>::zero()..<$t>::from_usize(3)
                );
                let (min, opt_max) = iter.size_hint();
                assert_eq!(min, 6, "size_hint min failed for type {}", stringify!($t));
                assert_eq!(opt_max, Some(6), "size_hint max failed for type {}", stringify!($t));
            }
        }
        )*
    }
}

impl_idx_for!(
    isize, usize, i128, u128, i64, u64, i32, u32, i16, u16, i8, u8
);

impl_idx_smoke_tests! {
    smoke_test_usize: usize,
    smoke_test_isize: isize,
    smoke_test_u128: u128,
    smoke_test_i128: i128,
    smoke_test_u64: u64,
    smoke_test_i64: i64,
    smoke_test_i32: i32,
    smoke_test_u32: u32,
    smoke_test_u16: u16,
    smoke_test_i16: i16,
    smoke_test_u8: u8,
    smoke_test_i8: i8
}
