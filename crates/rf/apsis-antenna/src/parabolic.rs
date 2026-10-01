//! The parabolic aperture model.
//!
//! 抛物面孔径模型。

use apsis_rf::Gain;
use core::f64::consts::PI;

/// A parabolic reflector described by its diameter and aperture efficiency.
///
/// The aperture efficiency collects every loss between the power delivered
/// to the feed and the power that leaves the aperture as a uniform plane
/// wave: illumination taper, spillover, blockage by the feed and its
/// supports, and surface error. A well adjusted dish reaches 0.65 to 0.70,
/// and a small or simply built one is nearer 0.55.
///
/// 由其直径与孔径效率描述的抛物面反射体。
///
/// 孔径效率汇集了从送到馈源的功率到以均匀平面波离开孔径的功率之间的
/// 每一项损耗：照射锥削、溢出、馈源及其支撑的遮挡，以及表面误差。调准
/// 良好的天线达到 0.65 至 0.70，小而简易的接近 0.55。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParabolicAperture {
    /// The reflector diameter, in meters.
    ///
    /// 反射面直径，单位为米。
    pub diameter: f64,
    /// The aperture efficiency, dimensionless, in `(0, 1]`.
    ///
    /// 孔径效率，无量纲，落在 `(0, 1]` 内。
    pub efficiency: f64,
}

impl ParabolicAperture {
    /// Returns the physical area of the aperture, in square meters.
    ///
    /// 返回孔径的物理面积，单位为平方米。
    pub fn area(self) -> f64 {
        PI * (self.diameter / 2.0).powi(2)
    }

    /// Returns the effective area, in square meters.
    ///
    /// This is the physical area scaled by the aperture efficiency. It is
    /// the area that, if it collected uniformly, would deliver the observed
    /// power.
    ///
    /// 返回有效面积，单位为平方米。
    ///
    /// 这是物理面积按孔径效率缩放后的值。它是这样一块面积：若均匀收集，
    /// 将给出所观测到的功率。
    pub fn effective_area(self) -> f64 {
        self.area() * self.efficiency
    }

    /// Returns `true` when the aperture describes a usable dish.
    ///
    /// 当该孔径描述一个可用天线时返回 `true`。
    pub fn is_valid(self) -> bool {
        self.diameter.is_finite()
            && self.diameter > 0.0
            && self.efficiency.is_finite()
            && self.efficiency > 0.0
            && self.efficiency <= 1.0
    }

    /// Returns the peak gain on boresight at the given wavelength.
    ///
    /// The relation is
    ///
    /// ```text
    /// G = eta (pi D / lambda)^2
    /// ```
    ///
    /// which follows from the effective area through
    /// [`Gain::from_effective_aperture`]. Returns `None` for an invalid
    /// aperture or wavelength.
    ///
    /// 返回给定波长处视轴上的峰值增益。
    ///
    /// 关系为
    ///
    /// ```text
    /// G = eta (pi D / lambda)^2
    /// ```
    ///
    /// 它由有效面积经 [`Gain::from_effective_aperture`] 得出。孔径或波长
    /// 无效时返回 `None`。
    pub fn peak_gain(self, wavelength: f64) -> Option<Gain> {
        if !self.is_valid() {
            return None;
        }
        Gain::from_effective_aperture(self.effective_area(), wavelength)
    }

    /// Returns the half power beam width at the given wavelength, in
    /// radians.
    ///
    /// This uses the uniform illumination relation, which is the narrowest
    /// a dish of this diameter can be. A real dish with an illumination
    /// taper is wider; use [`ParabolicAperture::half_power_beam_width`] to
    /// include the taper.
    ///
    /// Returns `None` for an invalid aperture or wavelength.
    ///
    /// 返回给定波长处的半功率波束宽度，单位为弧度。
    ///
    /// 这里用的是均匀照射关系，即该直径天线所能达到的最窄。带照射锥削
    /// 的实际天线更宽；要计入锥削请用
    /// [`ParabolicAperture::half_power_beam_width`]。
    ///
    /// 孔径或波长无效时返回 `None`。
    pub fn uniform_beam_width(self, wavelength: f64) -> Option<f64> {
        if !self.is_valid() {
            return None;
        }
        beam_width(self.diameter, wavelength, 1.02)
    }

    /// Returns the half power beam width at the given wavelength, in
    /// radians, allowing for an illumination taper.
    ///
    /// A tapered illumination reduces the sidelobes at the cost of a wider
    /// main lobe. The factor `k` in `theta = k lambda / D` rises from about
    /// 1.02 for uniform illumination to about 1.2 for the taper that
    /// minimises sidelobes. A value of 1.15 is representative of a
    /// well designed dish.
    ///
    /// Returns `None` for an invalid aperture or wavelength.
    ///
    /// 返回给定波长处的半功率波束宽度（弧度），并计入照射锥削。
    ///
    /// 锥削照射以加宽主瓣为代价降低旁瓣。`theta = k lambda / D` 中的因子
    /// `k` 从均匀照射的约 1.02 升到最小化旁瓣的锥削的约 1.2。1.15 是
    /// 设计良好的天线的代表值。
    ///
    /// 孔径或波长无效时返回 `None`。
    pub fn half_power_beam_width(self, wavelength: f64) -> Option<f64> {
        if !self.is_valid() {
            return None;
        }
        beam_width(self.diameter, wavelength, 1.15)
    }
}

/// Returns the half power beam width of a circular aperture, in radians.
///
/// The relation is `theta = k lambda / D`, where `k` depends on how the
/// aperture is illuminated. Uniform illumination gives the narrowest beam,
/// with `k` about 1.02, and the taper that suppresses sidelobes gives the
/// widest, with `k` about 1.2.
///
/// The result is the full width between the two three decibel points, not
/// the half width. It is a small angle approximation, so the caller should
/// confirm that the resulting width is small compared to a radian.
///
/// Returns `None` when the diameter or wavelength is not positive, or when
/// `k` is not positive.
///
/// 返回圆孔径的半功率波束宽度，单位为弧度。
///
/// 关系为 `theta = k lambda / D`，其中 `k` 取决于孔径如何被照射。均匀
/// 照射给出最窄波束，`k` 约为 1.02；抑制旁瓣的锥削给出最宽的，`k` 约为
/// 1.2。
///
/// 结果是两个三分贝点之间的全宽，而非半宽。它是小角近似，故调用方应
/// 确认所得宽度相对一弧度很小。
///
/// 直径或波长非正，或 `k` 非正时返回 `None`。
pub fn beam_width(diameter: f64, wavelength: f64, k: f64) -> Option<f64> {
    if !diameter.is_finite()
        || diameter <= 0.0
        || !wavelength.is_finite()
        || wavelength <= 0.0
        || !k.is_finite()
        || k <= 0.0
    {
        return None;
    }
    Some(k * wavelength / diameter)
}

/// Returns the angle at which the squared angle law of
/// [`off_axis_attenuation`](crate::off_axis_attenuation) reaches the
/// sidelobe floor.
///
/// Beyond this angle the model holds the gain at the floor rather than
/// letting it fall without bound. The angle follows from setting the
/// attenuation `12 (theta / theta_0)^2` equal to the floor, which gives
/// `theta = theta_0 sqrt(floor / 12)`.
///
/// Returns `None` when the beam width or floor is not positive and finite.
///
/// 返回 [`off_axis_attenuation`](crate::off_axis_attenuation) 的角平方律
/// 到达旁瓣底限的角度。
///
/// 超过该角度后，模型把增益保持在底限，而非任其无界下降。该角度由令
/// 衰减 `12 (theta / theta_0)^2` 等于底限得出，即
/// `theta = theta_0 sqrt(floor / 12)`。
///
/// 波束宽度或底限非正且非有限时返回 `None`。
pub fn floor_angle(beam_width: f64, floor_db: f64) -> Option<f64> {
    if !beam_width.is_finite() || beam_width <= 0.0 {
        return None;
    }
    if !floor_db.is_finite() || floor_db <= 0.0 {
        return None;
    }
    Some(beam_width * (floor_db / 12.0).sqrt())
}
