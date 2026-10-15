//! The foundation layer: shared types, units, mathematics, and macros.
//!
//! These modules sit at the left of the layer order, so every other layer may
//! use them.
//!
//! 基础层：共享类型、单位、数学与宏。
//!
//! 这些模块位于层序的最左端，故其余每一层都可以使用它们。

pub mod apsis_core;
pub mod apsis_macros;
pub mod apsis_math;
pub mod apsis_units;
