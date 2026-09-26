//! Internal helpers that generate the arithmetic operators for a quantity
//! newtype.
//!
//! The operators are mechanical, so they are generated from one definition
//! rather than repeated per unit. The macro is crate internal; the crate
//! documentation describes the public behaviour.

/// Implements `Add`, `Sub`, `Neg`, `Mul<f64>`, `Div<f64>`, `Mul<Self> for
/// f64`, and `Div<Self>` for a `f64` backed quantity newtype.
///
/// The newtype must be a tuple struct with a single `f64` field visible in
/// the invoking module.
macro_rules! impl_quantity_ops {
    ($quantity:ident) => {
        impl core::ops::Add for $quantity {
            type Output = Self;

            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl core::ops::Sub for $quantity {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl core::ops::Neg for $quantity {
            type Output = Self;

            fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl core::ops::Mul<f64> for $quantity {
            type Output = Self;

            fn mul(self, rhs: f64) -> Self {
                Self(self.0 * rhs)
            }
        }

        impl core::ops::Div<f64> for $quantity {
            type Output = Self;

            fn div(self, rhs: f64) -> Self {
                Self(self.0 / rhs)
            }
        }

        impl core::ops::Mul<$quantity> for f64 {
            type Output = $quantity;

            fn mul(self, rhs: $quantity) -> $quantity {
                $quantity(self * rhs.0)
            }
        }

        impl core::ops::Div<$quantity> for $quantity {
            type Output = f64;

            fn div(self, rhs: Self) -> f64 {
                self.0 / rhs.0
            }
        }
    };
}

pub(crate) use impl_quantity_ops;
