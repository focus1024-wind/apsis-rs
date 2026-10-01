//! Tests for the workspace error type and result alias.
//!
//! 工作区错误类型与结果别名的测试。

use apsis_core::{Error, ErrorKind, Result};
use core::fmt;

#[derive(Debug)]
struct Underlying;

impl fmt::Display for Underlying {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("underlying failure")
    }
}

impl core::error::Error for Underlying {}

#[test]
fn kind_is_reported() {
    let error = Error::new(ErrorKind::Parse);
    assert_eq!(error.kind(), ErrorKind::Parse);
    assert_eq!(error.message(), None);
}

#[test]
fn message_is_reported() {
    let error = Error::with_message(ErrorKind::OutOfRange, "semi-major axis is negative");
    assert_eq!(error.kind(), ErrorKind::OutOfRange);
    assert_eq!(error.message(), Some("semi-major axis is negative"));
}

#[test]
fn display_includes_kind_and_message() {
    let error = Error::with_message(ErrorKind::Parse, "bad checksum");
    assert_eq!(error.to_string(), "parse error: bad checksum");
    assert_eq!(Error::new(ErrorKind::NotFound).to_string(), "not found");
}

#[test]
fn source_is_exposed() {
    let error = Error::with_source(ErrorKind::Other, Underlying);
    assert!(std::error::Error::source(&error).is_some());
    assert_eq!(error.kind(), ErrorKind::Other);
}

#[test]
fn context_and_cause_replace_previous_values() {
    let error = Error::new(ErrorKind::Other)
        .with_context("first")
        .with_context("second")
        .caused_by(Underlying);
    assert_eq!(error.message(), Some("second"));
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn converts_from_kind() {
    let error: Error = ErrorKind::Unsupported.into();
    assert_eq!(error.kind(), ErrorKind::Unsupported);
}

#[test]
fn works_in_result_position() {
    fn fail() -> Result<u32> {
        Err(Error::new(ErrorKind::NotFinite))
    }

    let outcome = fail();
    assert!(outcome.is_err());
    assert_eq!(
        outcome.err().map(|error| error.kind()),
        Some(ErrorKind::NotFinite)
    );
}
