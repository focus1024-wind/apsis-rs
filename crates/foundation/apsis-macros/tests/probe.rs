//! Probe of the built in macro facilities.
//!
//! These tests record what `macro_rules!` can express for the workspace. They
//! are kept as executable documentation of the decision to leave this crate
//! empty for now: see the crate documentation.

#[test]
fn builtin_macros_can_substitute_a_type_and_a_literal() {
    // A `macro_rules!` macro can paste a type and a literal together, which
    // is the only form the workspace needs.
    macro_rules! quantity {
        ($quantity:ty, $value:literal) => {
            <$quantity>::new($value)
        };
    }

    // `f64` has no `new`, so the probe uses a type that does.
    struct Meters(f64);
    impl Meters {
        const fn new(value: f64) -> Self {
            Self(value)
        }
    }

    let value = quantity!(Meters, 1.5);
    assert_eq!(value.0, 1.5);
}

#[test]
fn builtin_macros_can_dispatch_on_a_trailing_token() {
    macro_rules! to_meters {
        ($value:literal m) => {
            $value
        };
        ($value:literal km) => {
            $value * 1_000.0
        };
    }

    assert_eq!(to_meters!(2.5 m), 2.5);
    assert_eq!(to_meters!(2.5 km), 2_500.0);
}
