//! Shared foundation types for the ApsisRS workspace: a general purpose
//! error type, result aliases, physical constants, and the unit marker
//! trait.
//!
//! This crate sits at the bottom of the dependency graph. It has no
//! dependency on any other ApsisRS crate, so every other crate may use it.
//!
//! Three concerns live here:
//!
//! - [`Error`] and [`Result`] are a general purpose error carrying a coarse
//!   [`ErrorKind`]. They exist so that a function whose failure has no
//!   interesting structure can report it without inventing a type, and so
//!   that the workspace has a shape to fall back on. They are deliberately
//!   *not* the error type of the layers: a layer with a domain specific
//!   failure defines its own `Error` and `ErrorKind`, whose variants name
//!   the concrete conditions a caller would want to match on. See
//!   `docs/design/errors.md` for why, and for the conventions the domain
//!   errors follow.
//! - [`constant`] holds physical and astronomical constants expressed in
//!   SI units.
//! - [`Quantity`] marks newtypes that carry a physical unit, so generic
//!   code can accept any quantity and still recover its unit type.
//!
//! ApsisRS 工作区共享的基础类型：一个通用错误类型、结果别名、物理常量，以及
//! 单位标记 trait。
//!
//! 本 crate 位于依赖图的底部。它不依赖任何其他 ApsisRS crate，故其余每个
//! crate 都可以使用它。
//!
//! 这里有三件事：
//!
//! - [`Error`] 与 [`Result`] 是携带粗糙 [`ErrorKind`] 的通用错误。它们存在，
//!   是为了让失败没有有趣结构的函数无需发明一个类型即可报告它，也是为了让
//!   工作区有一个可退守的形状。它们有意地*不是*各层的错误类型：具有领域
//!   特定失败的一层会定义自己的 `Error` 与 `ErrorKind`，其变体指名调用方
//!   想要匹配的具体情形。原因以及各领域错误遵循的约定见
//!   `docs/design/errors.md`。
//! - [`constant`] 保存以 SI 单位表达的物理与天文常量。
//! - [`Quantity`] 标记携带有物理单位的 newtype，使泛型代码可以接受任何量
//!   并仍能恢复其单位类型。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod constant;
mod error;
mod quantity;

pub use error::{Error, ErrorKind, Result};
pub use quantity::{Quantity, Unit};
