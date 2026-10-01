//! The Earth gravitational parameter.

/// The gravitational parameter of the central body, in cubic kilometers per
/// second squared.
///
/// Two values are offered because the workspace supports both gravity
/// models, and the correct choice is the one that matches the source of the
/// orbital elements. For elements derived from a two-line element set, the
/// WGS 72 value is the one the SGP4 theory was fitted to.
///
/// 中心天体的引力参数，单位为立方千米每二次方秒。
///
/// 提供两个值是因为工作区支持两种重力模型，正确选择取决于轨道根数的来源。对
/// 由两线根数集导出的根数，应取 WGS 72 值，即 SGP4 理论拟合所用的那一个。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityParameter(pub f64);

impl GravityParameter {
    /// The WGS 72 gravitational parameter, `398 600.8 km^3/s^2`.
    ///
    /// WGS 72 引力参数，`398 600.8 km^3/s^2`。
    pub const WGS72: Self = Self(398_600.8);

    /// The WGS 84 gravitational parameter, `398 600.5 km^3/s^2`.
    ///
    /// WGS 84 引力参数，`398 600.5 km^3/s^2`。
    pub const WGS84: Self = Self(398_600.5);

    /// Returns the value in cubic kilometers per second squared.
    ///
    /// 返回该值，单位为立方千米每二次方秒。
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Returns the square root of the parameter, in kilometers to the three
    /// halves per second.
    ///
    /// 返回该参数的平方根，单位为千米的三分之二次方每秒。
    pub fn sqrt(self) -> f64 {
        self.0.sqrt()
    }
}
