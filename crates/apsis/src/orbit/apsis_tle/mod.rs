//! Parsing of two-line element sets.
//!
//! A two-line element set (TLE) describes a satellite orbit in the form
//! used by the North American Aerospace Defense Command and distributed by
//! Space-Track. The name of the satellite is carried on an optional first
//! line, and the two following lines carry the orbital data.
//!
//! The format is fixed width. Every field occupies a fixed range of columns
//! and the checksum of each line is the sum of its digits modulo ten, where
//! a minus sign counts as one. This module parses a TLE into structured
//! fields and verifies the line checksums and the matching element set
//! number. It does not interpret the fields; the mean elements and their
//! units are the concern of `apsis_sgp4`.
//!
//! Column positions are one based, matching the documentation of the format.
//! Two conventions from the format are worth stating explicitly, because
//! they are easy to misread:
//!
//! - An exponent field is written as a signed single digit followed by the
//!   magnitude. For example ` 12345-3` means `0.12345e-3`, not `12345e-3`.
//! - An assumed decimal point is written without a decimal point. For
//!   example the eccentricity field `0006703` means `0.0006703`.
//!
//! Source: the field layout follows the format description published by
//! `CelesTrak`, *FAQs: Two-Line Element Set Format*, which documents the
//! columns defined by the original NORAD specification. The checksum rule
//! is the one stated there and used by Space-Track.
//!
//! ```
//! use apsis::apsis_tle::TwoLineElement;
//!
//! let text = concat!(
//!     "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\r\n",
//!     "2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537\r\n",
//! );
//! let element = TwoLineElement::parse(text).expect("sample is well formed");
//! assert_eq!(element.satellite_number, 25544);
//! assert!((element.eccentricity - 0.0006703).abs() < 1e-12);
//! ```
//!
//! 两行根数集的解析。
//!
//! 两行根数集（TLE）以北美防空司令部所用、由 Space-Track 分发的形式描述
//! 卫星轨道。卫星名称携带在可选的第一行，其后两行携带轨道数据。
//!
//! 该格式是定宽的。每个字段占据固定的列范围，每行的校验和是其各位数字之和
//! 模十，其中负号计为一。本模块把 TLE 解析为结构化字段，并校验各行校验和
//! 与相匹配的根数集编号。它不解释各字段；平根数及其单位是 `apsis_sgp4` 的
//! 事。
//!
//! 列位置以 1 为基，与该格式的文档一致。该格式有两条约定值得明确写出，因为
//! 它们容易误读：
//!
//! - 指数字段写为一个带符号的单数字后接幅度。例如 ` 12345-3` 意为
//!   `0.12345e-3`，而非 `12345e-3`。
//! - 隐含小数点不写小数点。例如偏心率字段 `0006703` 意为 `0.0006703`。
//!
//! 出处：字段布局遵循 `CelesTrak` 发布的格式说明，*FAQs: Two-Line Element Set
//! Format*，它记录了原 NORAD 规范所定义的各列。校验和规则即其中所述、
//! Space-Track 所用者。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod checksum;
mod element;
mod field;

pub use checksum::{line_checksum, verify_line_checksum};
pub use element::{Classification, TwoLineElement};
pub use field::{ElementError, ElementErrorKind, Result};
