//! Mission sequences and impulsive transfer planning.
//!
//! The module provides the two pieces an Astrogator-style analysis needs at
//! its base: a two-impulse Hohmann plan between circular orbits, and a
//! mission sequence that can propagate a state and apply impulsive velocity
//! changes in order. The sequence is the minimal mission control sequence:
//! segments run from the initial state, and every segment records an event
//! with its elapsed time and cost.
//!
//! 任务序列与脉冲转移规划。
//!
//! 本模块提供 Astrogator 式分析所需的两项基础能力：圆轨道之间的双脉冲
//! 霍曼方案，以及按次序传播状态并施加脉冲速度变化的任务序列。该序列即最小
//! 任务控制序列：各段从初始状态依次运行，每一段都记录一个带耗时与代价的事件。

use crate::apsis_kepler::{GravityParameter, KeplerianElements, State};
use crate::apsis_maneuver::{Impulse, ManeuverError, hohmann_transfer};
use crate::apsis_math::Vec3;

use crate::apsis_core::{Error, ErrorKind, Result};

/// A two-impulse transfer between two circular orbits.
///
/// 两个圆轨道之间的双脉冲转移。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HohmannPlan {
    /// Radius of the departure orbit, in kilometers.
    ///
    /// 出发轨道半径，单位为千米。
    pub from_radius_km: f64,
    /// Radius of the arrival orbit, in kilometers.
    ///
    /// 到达轨道半径，单位为千米。
    pub to_radius_km: f64,
    /// Semi-major axis of the transfer ellipse, in kilometers.
    ///
    /// 转移椭圆半长轴，单位为千米。
    pub transfer_semi_major_axis_km: f64,
    /// First burn, along the direction of motion, in kilometers per second.
    ///
    /// 第一次点火，沿运动方向，单位为千米每秒。
    pub first_burn_km_s: f64,
    /// Second burn, along the direction of motion, in kilometers per second.
    ///
    /// 第二次点火，沿运动方向，单位为千米每秒。
    pub second_burn_km_s: f64,
    /// Total delta-v, the sum of the magnitudes, in kilometers per second.
    ///
    /// 总速度变化，即两次点火大小之和，单位为千米每秒。
    pub total_delta_v_km_s: f64,
    /// Coast time between the burns, in seconds.
    ///
    /// 两次点火之间的滑行时间，单位为秒。
    pub transfer_time_seconds: f64,
    /// Whether the transfer raises the orbit.
    ///
    /// 转移是否抬升轨道。
    pub raises: bool,
}

/// Plans a Hohmann transfer under the WGS 84 gravity model.
///
/// 以 WGS 84 重力模型规划霍曼转移。
pub fn plan_hohmann(from_radius_km: f64, to_radius_km: f64) -> Result<HohmannPlan> {
    plan_hohmann_with(from_radius_km, to_radius_km, GravityParameter::WGS84)
}

/// Plans a Hohmann transfer under a chosen gravity model.
///
/// Returns an error when a radius is not usable or the two radii are equal.
///
/// 以选定重力模型规划霍曼转移。
///
/// 半径不可用或二者相等时返回错误。
pub fn plan_hohmann_with(
    from_radius_km: f64,
    to_radius_km: f64,
    parameter: GravityParameter,
) -> Result<HohmannPlan> {
    let transfer =
        hohmann_transfer(from_radius_km, to_radius_km, parameter).map_err(maneuver_error)?;
    Ok(HohmannPlan {
        from_radius_km: transfer.from_radius,
        to_radius_km: transfer.to_radius,
        transfer_semi_major_axis_km: transfer.transfer_semi_major_axis,
        first_burn_km_s: transfer.first_burn,
        second_burn_km_s: transfer.second_burn,
        total_delta_v_km_s: transfer.total_delta_v(),
        transfer_time_seconds: transfer.transfer_time,
        raises: transfer.is_raising(),
    })
}

/// One segment of a mission sequence.
///
/// 任务序列的一段。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    /// Two-body propagation for a duration, in seconds.
    ///
    /// 二体传播一段时长，单位为秒。
    Propagate {
        /// Duration in seconds.
        ///
        /// 时长，单位为秒。
        seconds: f64,
    },
    /// Instantaneous velocity change in the inertial frame, in km/s.
    ///
    /// 惯性系中的瞬时速度变化，单位为 km/s。
    Impulse {
        /// Velocity change vector.
        ///
        /// 速度变化矢量。
        delta_v_km_s: Vec3,
    },
}

/// The kind of a mission event.
///
/// 任务事件的类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// A propagation segment.
    ///
    /// 传播段。
    Propagate,
    /// An impulse segment.
    ///
    /// 脉冲段。
    Impulse,
}

/// One event produced by a mission sequence.
///
/// 任务序列产生的一个事件。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MissionEvent {
    /// Index of the segment that produced the event.
    ///
    /// 产生该事件的段下标。
    pub segment_index: usize,
    /// Elapsed time at the end of the segment, in seconds.
    ///
    /// 该段结束时的累计时间，单位为秒。
    pub elapsed_seconds: f64,
    /// Kind of the segment.
    ///
    /// 段的类型。
    pub kind: EventKind,
    /// Delta-v of an impulse event, in km/s.
    ///
    /// 脉冲事件的 delta-v，单位为 km/s。
    pub delta_v_km_s: Option<f64>,
}

/// The result of running a mission sequence.
///
/// 运行任务序列所得的结果。
#[derive(Debug, Clone, PartialEq)]
pub struct MissionResult {
    /// Elements at the end of the sequence.
    ///
    /// 序列结束时的根数。
    pub final_elements: KeplerianElements,
    /// Cartesian state at the end of the sequence.
    ///
    /// 序列结束时的笛卡尔状态。
    pub final_state: State,
    /// Total elapsed time in seconds.
    ///
    /// 总耗时，单位为秒。
    pub elapsed_seconds: f64,
    /// Total delta-v applied by the sequence, in km/s.
    ///
    /// 序列施加的总 delta-v，单位为 km/s。
    pub total_delta_v_km_s: f64,
    /// Events in execution order.
    ///
    /// 按执行次序排列的事件。
    pub events: Vec<MissionEvent>,
}

/// A minimal mission control sequence.
///
/// 最小任务控制序列。
#[derive(Debug, Clone, PartialEq)]
pub struct MissionSequence {
    /// Initial elements.
    ///
    /// 初始根数。
    pub initial: KeplerianElements,
    /// Gravity model used by propagation and the element conversion.
    ///
    /// 传播与根数换算所用的重力模型。
    pub parameter: GravityParameter,
    segments: Vec<Segment>,
}

impl MissionSequence {
    /// Creates an empty sequence from valid initial elements.
    ///
    /// Returns an error when the elements are not a valid elliptic orbit.
    ///
    /// 由有效初始根数创建空序列。
    ///
    /// 根数不是有效椭圆轨道时返回错误。
    pub fn new(initial: KeplerianElements, parameter: GravityParameter) -> Result<Self> {
        if !initial.is_valid() {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "initial elements must describe an elliptic orbit".to_string(),
            ));
        }
        Ok(Self {
            initial,
            parameter,
            segments: Vec::new(),
        })
    }

    /// Appends a propagation segment.
    ///
    /// Returns an error when the duration is not positive and finite.
    ///
    /// 追加一个传播段。
    ///
    /// 时长非正或非有限时返回错误。
    pub fn propagate(mut self, seconds: f64) -> Result<Self> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "propagation duration must be positive and finite".to_string(),
            ));
        }
        self.segments.push(Segment::Propagate { seconds });
        Ok(self)
    }

    /// Appends an impulsive segment.
    ///
    /// Returns an error when the change is zero or not finite.
    ///
    /// 追加一个脉冲段。
    ///
    /// 变化为零或非有限时返回错误。
    pub fn impulse(mut self, delta_v_km_s: Vec3) -> Result<Self> {
        Impulse::new(delta_v_km_s).map_err(maneuver_error)?;
        self.segments.push(Segment::Impulse { delta_v_km_s });
        Ok(self)
    }

    /// Returns the segments in execution order.
    ///
    /// 按执行次序返回各段。
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Runs the sequence and returns the final state and events.
    ///
    /// Returns an error when a segment cannot be applied, which happens for
    /// a non-elliptic intermediate orbit.
    ///
    /// 运行序列并返回末状态与事件。
    ///
    /// 某段无法施加（中间轨道非椭圆）时返回错误。
    pub fn run(&self) -> Result<MissionResult> {
        let mu = self.parameter.value();
        let mut elements = self.initial;
        let mut elapsed = 0.0;
        let mut total_delta_v = 0.0;
        let mut events = Vec::with_capacity(self.segments.len());
        for (index, segment) in self.segments.iter().enumerate() {
            match *segment {
                Segment::Propagate { seconds } => {
                    let mean_motion = (mu / elements.semi_major_axis.powi(3)).sqrt();
                    let mean_anomaly = elements.mean_anomaly() + mean_motion * seconds;
                    elements = elements.with_mean_anomaly(mean_anomaly);
                    elapsed += seconds;
                    events.push(MissionEvent {
                        segment_index: index,
                        elapsed_seconds: elapsed,
                        kind: EventKind::Propagate,
                        delta_v_km_s: None,
                    });
                }
                Segment::Impulse { delta_v_km_s } => {
                    let state = elements.to_state(self.parameter).ok_or_else(|| {
                        Error::with_message(
                            ErrorKind::InvalidInput,
                            "intermediate elements are not a valid elliptic orbit".to_string(),
                        )
                    })?;
                    let impulse = Impulse::new(delta_v_km_s).map_err(maneuver_error)?;
                    let changed = State {
                        position: state.position,
                        velocity: impulse.apply_to_velocity(state.velocity),
                    };
                    elements = KeplerianElements::from_state(&changed, self.parameter).ok_or_else(
                        || {
                            Error::with_message(
                                ErrorKind::InvalidInput,
                                "the post-burn state is not an elliptic orbit".to_string(),
                            )
                        },
                    )?;
                    total_delta_v += impulse.magnitude();
                    events.push(MissionEvent {
                        segment_index: index,
                        elapsed_seconds: elapsed,
                        kind: EventKind::Impulse,
                        delta_v_km_s: Some(impulse.magnitude()),
                    });
                }
            }
        }
        let final_state = elements.to_state(self.parameter).ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidInput,
                "final elements are not a valid elliptic orbit".to_string(),
            )
        })?;
        Ok(MissionResult {
            final_elements: elements,
            final_state,
            elapsed_seconds: elapsed,
            total_delta_v_km_s: total_delta_v,
            events,
        })
    }
}

fn maneuver_error(error: ManeuverError) -> Error {
    Error::with_message(ErrorKind::InvalidInput, error.to_string())
}
