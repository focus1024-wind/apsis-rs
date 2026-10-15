//! Macros that support `ApsisRS`.
//!
//! This module currently defines no macros. It is re-exported as
//! `apsis::apsis_macros` so that the public path stays available and a shared
//! macro has a place to live if one is ever needed.
//!
//! The decision to leave it empty follows from what the crate actually
//! needs. Rust's built in `macro_rules!` can already do everything the
//! foundation layer requires:
//!
//! - Substituting a type and a literal to build a quantity, as
//!   `quantity!(Meters, 1.5)`.
//! - Dispatching on a trailing token to accept a unit suffix, as
//!   `to_meters!(2.5 km)`.
//!
//! Both forms are demonstrated by the tests under
//! `tests/foundation/apsis_macros/`. They can be written directly in the
//! module that uses them, so no shared macro module is needed.
//!
//! Two limits are worth recording, because they explain why a macro based
//! unit system is not provided:
//!
//! - A `macro_rules!` macro cannot be generic over a unit suffix that is not
//!   an identifier. `1.5 m` and `1.5 km` are accepted by separate rules, so
//!   the macro must enumerate every suffix it supports. The unit newtypes in
//!   `apsis_units` already give compile time unit safety without that list,
//!   so the macro would add surface area without adding checking.
//! - A declarative macro cannot inspect the value of its arguments, so it
//!   cannot fold a unit conversion at compile time beyond what constant
//!   evaluation already does. A literal such as `2.5 * 1000.0` is folded by
//!   the compiler anyway.
//!
//! If the crate later needs macros that generate items, such as deriving a
//! set of conversions from one definition, that is the point at which this
//! module should gain its first macro.
//!
//! 支持 `ApsisRS` 的宏。
//!
//! 本模块当前不定义任何宏。它以 `apsis::apsis_macros` 再导出，使该公开
//! 路径继续可用，并在需要一个共享宏时有存放之处。
//!
//! 让它保持为空的决定源于本模块实际所需。Rust 内建的 `macro_rules!` 已能完成
//! 基础层所需的一切：
//!
//! - 替换一个类型与一个字面量以构建量，如 `quantity!(Meters, 1.5)`。
//! - 依据尾随记号分派以接受单位后缀，如 `to_meters!(2.5 km)`。
//!
//! 两种形式都由 `tests/foundation/apsis_macros/` 下的测试演示。它们可以直接
//! 写在用到它们的模块中，故无需共享的宏模块。
//!
//! 有两点限制值得记下，因为它们解释了为何不提供基于宏的单位系统：
//!
//! - `macro_rules!` 宏不能对不是标识符的单位后缀泛型化。`1.5 m` 与
//!   `1.5 km` 由不同的规则接受，故该宏必须枚举它所支持的每个后缀。
//!   `apsis_units` 中的单位 newtype 已在没有该列表的情况下给出编译期单位
//!   安全，故该宏只会增加表面积而不增加检查。
//! - 声明式宏无法检视其参数的值，故它无法在常量求值已做的之外于编译期折叠
//!   单位换算。诸如 `2.5 * 1000.0` 这样的字面量反正会被编译器折叠。
//!
//! 若本模块日后需要生成项（例如从一个定义派生一组换算）的宏，那便是本
//! 模块应获得其第一个宏之时。

#![forbid(unsafe_code)]
#![deny(missing_docs)]
