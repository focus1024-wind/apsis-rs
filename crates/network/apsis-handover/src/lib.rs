//! Handover decision logic for mobile terminals.
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
//!
//! 移动终端的切换决策逻辑。
//!
//! 能被不止一颗卫星服务的终端必须选择其一，并在当前服务星变得不合适时改变其
//! 选择。本 crate 作出该决策。它不计算可见性窗口，那属于可见性层；它接收一份带有
//! 各自质量的候选卫星列表，并决定是否切换。
//!
//! 有两种效果相互对抗，二者都必须纳入策略：
//!
//! - **切换太迟**会掉链。若当前服务星跌到终端的最低值以下，链路在切换发生之前即
//!   失效。
//! - **切换太频繁**浪费容量与时间。每次切换都要付出信令代价并中断数据流，故在每个
//!   小起伏上都切换的终端表现很差，即便每个单独的选择看起来更好。
//!
//! 故该策略有三部分：当前服务星被视为不可用的阈值、候选在值得切换之前必须超出
//! 当前服务星的余量，以及两次切换之间的最短时间。该余量正是抑制振荡者，也是该
//! 决策的滞回。
//!
//! 出处：阈值、余量与驻留时间的结构即蜂窝网与卫星网中切换所用者，例如见 ITU-R
//! 建议书 M.1184 关于移动卫星系统技术特性的内容，以及移动无线电中切换滞回的一般
//! 论述。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod decision;
mod error;
mod policy;

pub use decision::{Candidate, HandoverDecision, decide, decide_with_history};
pub use error::{HandoverError, HandoverErrorKind};
pub use policy::{HandoverPolicy, HandoverRecord, HandoverTracker};
