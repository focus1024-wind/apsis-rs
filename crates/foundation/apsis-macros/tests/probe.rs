//! Probe of the built in macro facilities.
//!
//! These tests record what `macro_rules!` can express for the workspace. They
//! are kept as executable documentation of the decision to leave this crate
//! empty for now: see the crate documentation.
//!
//! 内建宏设施探查。
//!
//! 这些测试记录 `macro_rules!` 能为工作区表达什么。它们被保留为「目前让本 crate
//! 保持为空」这一决定的可执行文档：见 crate 文档。

#[test]
fn builtin_macros_can_substitute_a_type_and_a_literal() {
    // A `macro_rules!` macro can paste a type and a literal together, which
    // is the only form the workspace needs.
    // `macro_rules!` 宏可以把一个类型与一个字面量拼在一起，这是本工作区唯一需要的
    // 形式。
    macro_rules! quantity {
        ($quantity:ty, $value:literal) => {
            <$quantity>::new($value)
        };
    }

    // `f64` has no `new`, so the probe uses a type that does.
    // `f64` 没有 `new`，故本探查用一个有的类型。
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
