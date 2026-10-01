//! Handover decision logic for mobile terminals.
//!
//! 移动终端的切换决策逻辑。
//!
//! A terminal that can be served by more than one satellite must choose one,
//! and it must change its choice when the current server becomes unsuitable.
//! This crate makes that decision. It does not compute access windows, which
//! belong to the access layer; it takes a list of candidate satellites with
//! their quality and decides whether to hand over.
//!
//! Two effects oppose each other and both must be in the policy:
//!
//! - **Handing over too late** loses the link. If the current server drops
//!   below the terminal's minimum, the link fails before the switch happens.
//! - **Handing over too often** wastes capacity and time. Each handover costs
//!   signalling and interrupts the data flow, so a terminal that switches on
//!   every small fluctuation performs poorly even though each individual
//!   choice looked better.
//!
//! The policy therefore has three parts: a threshold at which the current
//! server is considered unusable, a margin by which a candidate must beat
//! the current server before it is worth switching to, and a minimum time
//! between handovers. The margin is what suppresses oscillation, and it is
//! the hysteresis of the decision.
//!
//! Source: the threshold, margin, and dwell time structure is the one used
//! for handover in cellular and satellite networks, as described for example
//! in ITU-R Recommendation M.1184 on the technical and operational
//! characteristics of mobile satellite systems, and in the general treatment
//! of handover hysteresis in mobile radio.
//!
//! ```
//! use apsis_handover::{Candidate, HandoverPolicy, HandoverDecision, decide};
//!
//! let policy = HandoverPolicy {
//!     minimum_quality: 0.2,
//!     handover_margin: 0.1,
//!     minimum_dwell_seconds: 5.0,
//! };
//!
//! // The current server is fine and the alternative is only slightly
//! // better, so nothing happens.
//! let candidates = [
//!     Candidate::new(1, 0.8, 100.0),
//!     Candidate::new(2, 0.85, 90.0),
//! ];
//! let decision = decide(&policy, Some(1), &candidates).expect("valid input");
//! assert_eq!(decision, HandoverDecision::Stay);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod decision;
mod error;
mod policy;

pub use decision::{Candidate, HandoverDecision, decide, decide_with_history};
pub use error::{HandoverError, HandoverErrorKind};
pub use policy::{HandoverPolicy, HandoverRecord, HandoverTracker};
