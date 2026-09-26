//! Marker traits for quantities that carry a physical unit.
//!
//! A quantity newtype stores a numeric value together with its unit in the
//! type. The [`Quantity`] trait exposes that unit through an associated
//! type, so generic code can be written over any quantity and still recover
//! which unit it was given.
//!
//! The unit itself is any type implementing [`Unit`]. A unit knows how many
//! of its kind are in the corresponding base unit, which allows conversion
//! between units of the same dimension. The base unit of a dimension always
//! has a factor of one.

use core::fmt;

/// A unit of a single physical dimension.
///
/// Implementors are zero sized marker types. Two units that implement
/// `Unit` for the same dimension convert through
/// [`Quantity::to_unit`](crate::Quantity::to_unit).
pub trait Unit: Copy + fmt::Debug + fmt::Display + 'static {
    /// The physical dimension this unit measures, such as length or time.
    type Dimension: fmt::Debug + 'static;

    /// The number of base units in one of this unit.
    ///
    /// For example a kilometer has a factor of 1000 when the base unit is
    /// the meter. The base unit of a dimension has a factor of exactly one.
    fn factor_to_base() -> f64;

    /// The symbol used to print the unit, such as `m` or `s`.
    fn symbol() -> &'static str;
}

/// A value paired with a unit.
///
/// Implementors are newtypes over `f64` and a [`Unit`]. The trait gives
/// generic code access to the stored value and its unit without assuming
/// which newtype it was handed.
pub trait Quantity: Copy {
    /// The unit this quantity is expressed in.
    type U: Unit;

    /// Returns the stored numeric value in this quantity's own unit.
    fn value(self) -> f64;

    /// Builds a quantity from a numeric value in this quantity's own unit.
    fn from_value(value: f64) -> Self;

    /// Converts this quantity to another unit of the same dimension.
    ///
    /// The value is scaled by the ratio of the two unit factors. The
    /// conversion is exact only when the factors are exact powers of ten,
    /// so callers that need a specific unit for output should convert once
    /// at the boundary rather than repeatedly.
    fn to_unit<U>(self) -> U
    where
        U: Quantity,
        <U as Quantity>::U: Unit<Dimension = <Self::U as Unit>::Dimension>,
    {
        let base = self.value() * Self::U::factor_to_base();
        U::from_value(base / <U as Quantity>::U::factor_to_base())
    }
}

#[cfg(test)]
mod tests {
    use super::{Quantity, Unit};
    use core::fmt;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Length;

    #[derive(Debug, Clone, Copy)]
    struct Meter;

    #[derive(Debug, Clone, Copy)]
    struct Kilometer;

    impl fmt::Display for Meter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("m")
        }
    }

    impl fmt::Display for Kilometer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("km")
        }
    }

    impl Unit for Meter {
        type Dimension = Length;

        fn factor_to_base() -> f64 {
            1.0
        }

        fn symbol() -> &'static str {
            "m"
        }
    }

    impl Unit for Kilometer {
        type Dimension = Length;

        fn factor_to_base() -> f64 {
            1_000.0
        }

        fn symbol() -> &'static str {
            "km"
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Meters(f64);

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Kilometers(f64);

    impl Quantity for Meters {
        type U = Meter;

        fn value(self) -> f64 {
            self.0
        }

        fn from_value(value: f64) -> Self {
            Self(value)
        }
    }

    impl Quantity for Kilometers {
        type U = Kilometer;

        fn value(self) -> f64 {
            self.0
        }

        fn from_value(value: f64) -> Self {
            Self(value)
        }
    }

    #[test]
    fn value_round_trips() {
        let distance = Meters::from_value(1_500.0);
        assert_eq!(distance.value(), 1_500.0);
    }

    #[test]
    fn converts_between_units_of_one_dimension() {
        let distance = Meters::from_value(2_500.0);
        let in_kilometers: Kilometers = distance.to_unit();
        assert!((in_kilometers.value() - 2.5).abs() < 1e-12);
    }

    #[test]
    fn conversion_round_trips() {
        let original = Kilometers::from_value(7.25);
        let round_tripped: Kilometers = original.to_unit::<Meters>().to_unit();
        assert!((round_tripped.value() - 7.25).abs() < 1e-12);
    }

    #[test]
    fn unit_metadata_is_available() {
        assert_eq!(<Meter as Unit>::symbol(), "m");
        assert_eq!(<Kilometer as Unit>::factor_to_base(), 1_000.0);
    }
}
