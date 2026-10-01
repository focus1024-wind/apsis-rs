//! The combined orbit type.

use apsis_ephemeris::EphemerisError;
use apsis_kepler::{GravityParameter, KeplerianElements};
use apsis_maneuver::ManeuverError;
use apsis_math::Vec3;
use apsis_sgp4::{Elements, GravityModel, PropagationError, Satellite, State};
use apsis_tle::TwoLineElement;
use core::fmt;

/// A satellite orbit, held as mean elements ready for propagation.
///
/// The type pairs a prepared SGP4 satellite with the gravity model it was
/// initialised against, because the osculating element conversion needs the
/// same gravitational parameter to be consistent with the propagation.
///
/// 以平均根数形式持有、随时可传播的卫星轨道。
///
/// 该类型把准备好的 SGP4 卫星与其初始化时所用的重力模型配对，因为密切根数
/// 换算需要同一个引力参数才能与传播保持一致。
#[derive(Debug, Clone)]
pub struct Orbit {
    satellite: Satellite,
    parameter: GravityParameter,
}

impl Orbit {
    /// Prepares an orbit from a parsed element set and a gravity model.
    ///
    /// 由已解析的根数集与重力模型准备轨道。
    ///
    /// The gravity model must match the one the element set was fitted
    /// against, which for a catalogued satellite means
    /// [`GravityModel::WGS72`]. Returns `None` when the elements are not
    /// propagatable.
    ///
    /// 由已解析的根数集与重力模型准备轨道。
    ///
    /// 重力模型须与该根数集拟合时所用者一致；对编目卫星即 [`GravityModel::WGS72`]。
    /// 当根数不可传播时返回 `None`。
    pub fn from_element(element: &TwoLineElement, model: GravityModel) -> Option<Self> {
        Self::try_from_element(element, model).ok()
    }

    /// Prepares an orbit from a parsed element set and a gravity model.
    ///
    /// 由已解析的根数集与重力模型准备轨道。
    ///
    /// Returns an error describing which element is not propagatable.
    ///
    /// 由已解析的根数集与重力模型准备轨道。
    ///
    /// 返回一个说明哪个根数不可传播的错误。
    pub fn try_from_element(
        element: &TwoLineElement,
        model: GravityModel,
    ) -> Result<Self, PropagationError> {
        let satellite = Satellite::try_from_element(element, model)?;
        Ok(Self {
            satellite,
            parameter: parameter_for(model),
        })
    }

    /// Prepares an orbit from mean elements already in model units.
    ///
    /// 由已按模型单位表示的平均根数准备轨道。
    pub fn from_elements(
        elements: &Elements,
        model: GravityModel,
    ) -> Result<Self, PropagationError> {
        let satellite = Satellite::try_from_elements(elements, model)?;
        Ok(Self {
            satellite,
            parameter: parameter_for(model),
        })
    }

    /// Returns the underlying propagator.
    ///
    /// 返回底层的传播器。
    pub fn satellite(&self) -> &Satellite {
        &self.satellite
    }

    /// Returns the gravitational parameter paired with the gravity model.
    ///
    /// The value is the one used for the osculating element conversion, so
    /// that the conversion is consistent with the propagation.
    ///
    /// 返回与该重力模型配对的引力参数。
    ///
    /// 该值即密切根数换算所用者，从而使换算与传播保持一致。
    pub fn gravity_parameter(&self) -> GravityParameter {
        self.parameter
    }

    /// Returns the epoch of the elements as a Modified Julian Date on the
    /// UTC scale.
    ///
    /// The epoch is the value written in the element set, which is the
    /// origin the propagator measures time from.
    ///
    /// 返回根数历元，为 UTC 标度上的修正儒略日（Modified Julian Date）。
    ///
    /// 该历元即根数集中写下的值，也是传播器度量的时间原点。
    pub fn epoch_mjd(&self) -> f64 {
        self.satellite.epoch_mjd()
    }

    /// Returns `true` when the propagator uses the deep space branch.
    ///
    /// 当传播器使用深空分支时返回 `true`。
    pub fn is_deep_space(&self) -> bool {
        self.satellite.is_deep_space()
    }

    /// Returns the state at a number of minutes from the element epoch.
    ///
    /// Position is in kilometers and velocity in kilometers per second, in
    /// the True Equator Mean Equinox frame.
    ///
    /// 返回距根数历元若干分钟处的状态。
    ///
    /// 位置单位为千米，速度单位为千米每秒，位于真赤道平春分点系。
    pub fn propagate_minutes(&self, minutes: f64) -> Result<State, PropagationError> {
        self.satellite.propagate_minutes(minutes)
    }

    /// Returns the state at a Modified Julian Date on the UTC scale.
    ///
    /// 返回 UTC 标度上某修正儒略日（Modified Julian Date）处的状态。
    pub fn propagate_at(&self, utc_mjd: f64) -> Result<State, PropagationError> {
        self.propagate_minutes((utc_mjd - self.satellite.epoch_mjd()) * 1_440.0)
    }

    /// Returns the osculating Keplerian elements at a number of minutes from
    /// the element epoch.
    ///
    /// The osculating elements describe the two-body orbit that matches the
    /// position and velocity at that instant. They are not the mean elements
    /// of the element set, and they change along the orbit because SGP4
    /// includes perturbations that the two-body model does not.
    ///
    /// Returns an error when the state cannot be propagated, or when the
    /// instantaneous orbit is not elliptical, which can happen for a state
    /// the propagation has pushed onto an escape trajectory.
    ///
    /// 返回距根数历元若干分钟处的密切开普勒根数。
    ///
    /// 密切根数描述与该瞬时位置和速度相匹配的二体轨道。它不是根数集的平均根数，
    /// 且沿轨道变化，因为 SGP4 含有二体模型所没有的摄动。
    ///
    /// 当状态无法传播，或瞬时轨道非椭圆时返回错误；后者可发生于传播把状态推上
    /// 逃逸轨迹的情形。
    pub fn osculating_elements(&self, minutes: f64) -> Result<KeplerianElements, PropagationError> {
        let state = self.propagate_minutes(minutes)?;
        // The two crates define their own state type, so the components are
        // transferred explicitly rather than relying on a shared type.
        // 两个 crate 各自定义状态类型，故显式转移各分量，而不依赖共享类型。
        let kepler_state = apsis_kepler::State {
            position: state.position,
            velocity: state.velocity,
        };
        KeplerianElements::from_state(&kepler_state, self.parameter).ok_or_else(|| {
            PropagationError::new(apsis_sgp4::PropagationErrorKind::OutOfRange)
                .with_detail("the instantaneous orbit is not elliptical")
        })
    }

    /// Returns the magnitude of the state vector.
    ///
    /// This is a convenience for the common case of wanting only the
    /// distance from the center of the Earth.
    ///
    /// 返回状态矢量的模。
    ///
    /// 这为只需到地心距离的常见情形提供便利。
    pub fn radius_at(&self, minutes: f64) -> Result<f64, PropagationError> {
        self.propagate_minutes(minutes)
            .map(|state| state.position.norm())
    }

    /// Returns the position at a number of minutes from the element epoch.
    ///
    /// 返回距根数历元若干分钟处的位置。
    pub fn position_at(&self, minutes: f64) -> Result<Vec3, PropagationError> {
        self.propagate_minutes(minutes).map(|state| state.position)
    }

    /// Samples the orbit into an ephemeris over a range of minutes.
    ///
    /// The propagation is the expensive part of a track, so a table built
    /// once and queried many times is worth the memory. The sample times are
    /// minutes from the element epoch, matching the rest of this type, while
    /// the returned ephemeris keeps seconds, matching `apsis-ephemeris`.
    ///
    /// The table is built with position and velocity from the propagator,
    /// so the Hermite interpolation has exact endpoint slopes and the error
    /// between samples is tiny; there is no need to use the position only
    /// route except for a source that has no velocity.
    ///
    /// Returns an error when the grid is not usable or when a sample cannot
    /// be propagated, which happens when the elements have decayed or the
    /// range leaves the region where the model is defined.
    ///
    /// 把该轨道在若干分钟的区间上采样为星历。
    ///
    /// 传播是生成轨迹中开销最大的部分，故一次建表、多次查询值得占用内存。采样
    /// 时刻以距根数历元的分钟计，与本类型其余部分一致；返回的星历则用秒，与
    /// `apsis-ephemeris` 一致。
    ///
    /// 该表用传播器给出的位置与速度建立，故 Hermite 插值具有精确的端点斜率，
    /// 样本间误差极小；除数据源没有速度外，无需使用仅位置的路径。
    ///
    /// 当网格不可用或某样本无法传播时返回错误；后者发生于根数已衰减，或区间
    /// 超出模型定义范围的情形。
    pub fn sample_ephemeris(
        &self,
        start_minutes: f64,
        end_minutes: f64,
        step_minutes: f64,
    ) -> Result<apsis_ephemeris::Ephemeris, EphemerisSamplingError> {
        let sampler = apsis_ephemeris::Sampler::new(
            start_minutes * 60.0,
            end_minutes * 60.0,
            step_minutes * 60.0,
        );
        if !sampler.is_valid() {
            return Err(EphemerisSamplingError::InvalidGrid);
        }
        let mut failure = None;
        let ephemeris = sampler.sample(|seconds| {
            // The minutes are recovered from the seconds the sampler uses,
            // so a non-integer step stays consistent with the grid.
            // 由采样器所用的秒数反推分钟，使非整数步长仍与网格一致。
            match self.propagate_minutes(seconds / 60.0) {
                Ok(state) => apsis_ephemeris::Sample::new(seconds, state.position, state.velocity),
                Err(error) => {
                    // The sampler cannot report the propagation failure, so
                    // it is recorded and the sample is filled with a value
                    // the table constructor will then reject.
                    // 采样器无法报告传播失败，故在此记录，并用表格构造器随后
                    // 会拒绝的值填充该样本。
                    failure = Some(error);
                    apsis_ephemeris::Sample::new(
                        seconds,
                        Vec3::new(f64::NAN, f64::NAN, f64::NAN),
                        Vec3::new(f64::NAN, f64::NAN, f64::NAN),
                    )
                }
            }
        });
        if let Some(error) = failure {
            return Err(EphemerisSamplingError::Propagation(error));
        }
        ephemeris.map_err(EphemerisSamplingError::Ephemeris)
    }

    /// Applies a finite burn and returns the resulting osculating elements.
    ///
    /// The burn is applied to the oscillating state at the given time, so the
    /// manoeuvre is described relative to the orbit the satellite is
    /// actually on rather than the mean elements. The resulting elements are
    /// the osculating elements of the post burn state, which is what a
    /// following propagation needs. Because the underlying propagator is
    /// built for one element set, this is a planning result rather than a
    /// new propagator; a caller that wants to propagate the new orbit starts
    /// a fresh `Orbit` from the returned elements.
    ///
    /// 施加有限推力点火并返回所得的密切根数。
    ///
    /// 点火施加于给定时刻的密切状态，故该机动是相对卫星实际所在轨道而非平均
    /// 根数描述的。所得根数是点火后状态的密切根数，正是后续传播所需的。由于
    /// 底层传播器是为一个根数集构建的，这给出的是规划结果而非新的传播器；
    /// 需要传播新轨道的调用方应据此创建新的 `Orbit`。
    ///
    /// 持续时间可由 30 天星历采样路径支撑。
    pub fn apply_burn(
        &self,
        minutes: f64,
        burn: apsis_maneuver::FiniteBurn,
        steps: usize,
    ) -> Result<KeplerianElements, ManeuverApplicationError> {
        let state = self
            .propagate_minutes(minutes)
            .map_err(ManeuverApplicationError::Propagation)?;
        let kepler_state = apsis_kepler::State {
            position: state.position,
            velocity: state.velocity,
        };
        let outcome = burn
            .apply(kepler_state, self.parameter, steps)
            .map_err(ManeuverApplicationError::Maneuver)?;
        KeplerianElements::from_state(&outcome.state, self.parameter)
            .ok_or(ManeuverApplicationError::NotElliptical)
    }
}

/// A failure while sampling an orbit into an ephemeris.
///
/// 把轨道采样为星历时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EphemerisSamplingError {
    /// The requested grid is not usable: the step is not positive or the
    /// range is empty or reversed.
    ///
    /// 请求的网格不可用：步长不为正，或区间为空或倒置。
    InvalidGrid,
    /// A sample could not be propagated.
    ///
    /// 某样本无法传播。
    Propagation(PropagationError),
    /// The table rejected the samples, which should not happen for a valid
    /// grid and is reported rather than hidden.
    ///
    /// 表格拒绝了这些样本；对有效网格这不应发生，故予以报告而非隐藏。
    Ephemeris(EphemerisError),
}

impl fmt::Display for EphemerisSamplingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGrid => f.write_str("the sampling grid is not usable"),
            Self::Propagation(error) => write!(f, "a sample could not be propagated: {error}"),
            Self::Ephemeris(error) => write!(f, "the ephemeris rejected the samples: {error}"),
        }
    }
}

impl core::error::Error for EphemerisSamplingError {}

/// A failure while applying a burn to an orbit.
///
/// 把点火作用于轨道时发生的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManeuverApplicationError {
    /// The state before the burn could not be propagated.
    ///
    /// 点火前的状态无法传播。
    Propagation(PropagationError),
    /// The burn itself was not usable.
    ///
    /// 点火本身不可用。
    Maneuver(ManeuverError),
    /// The post burn state is not an elliptical orbit, which happens when
    /// the burn is large enough to escape.
    ///
    /// 点火后状态不是椭圆轨道，这发生于点火足够大以致逃逸时。
    NotElliptical,
}

impl fmt::Display for ManeuverApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Propagation(error) => write!(f, "the state could not be propagated: {error}"),
            Self::Maneuver(error) => write!(f, "the burn is not usable: {error}"),
            Self::NotElliptical => f.write_str("the post burn orbit is not elliptical"),
        }
    }
}

impl core::error::Error for ManeuverApplicationError {}

/// Returns the gravitational parameter that matches a gravity model.
///
/// The two models differ slightly in their value of the gravitational
/// parameter, and using the wrong one shifts the osculating semi-major axis
/// by a small amount. The values are taken from the model definitions rather
/// than duplicated, so a change to a model propagates here.
///
/// 返回与某重力模型匹配的引力参数。
///
/// 两种模型的引力参数值略有差异，用错会使密切半长轴发生小幅偏移。这些值取自
/// 模型定义而非重复抄写，故模型的改动会传播到这里。
fn parameter_for(model: GravityModel) -> GravityParameter {
    // The two parameters differ by 0.3 km^3/s^2, about one part in a
    // million. The comparison is on the model's defining value, which is
    // exact for the constants defined in this workspace.
    // 两种参数相差 0.3 km^3/s^2，约为百万分之一。比较基于模型定义值，对本
    // 工作区定义的常量是精确的。
    if model.mu == GravityModel::WGS84.mu {
        GravityParameter::WGS84
    } else {
        GravityParameter::WGS72
    }
}
