//! Off-axis gain patterns.

use crate::parabolic::ParabolicAperture;
use apsis_rf::{Decibel, Gain};
use core::f64::consts::PI;

/// Returns the gain of an antenna at an angle from boresight, given a peak
/// gain and a half power beam width.
///
/// This is the squared angle model that several interference assessments
/// use, expressed in terms of the half power beam width so that any antenna
/// can be described by the two numbers a link budget has to hand. In
/// decibels,
///
/// ```text
/// G(theta) = G_peak - min(12 (theta / theta_0)^2, G_floor)
/// ```
///
/// where `theta_0` is the half power beam width and `G_floor` is the
/// sidelobe level to fall to, in decibels below the peak. The squared angle
/// law comes from the first term of the Airy pattern of a uniformly
/// illuminated aperture, which is a good approximation down to the first
/// sidelobe.
///
/// The coefficient twelve is chosen so that the attenuation reaches three
/// decibels at `theta_0 / 2`, which is the definition of the half power beam
/// width. A pattern taken from a recommendation should be used in place of
/// this one where the recommendation applies: see
/// [`reference_pattern_gain`](crate::reference_pattern_gain) for the ITU-R
/// S.465 pattern, which differs from this model in the main lobe and also in
/// the shape of its sidelobe region.
///
/// Returns `None` when the peak gain has no decibel form, or when the beam
/// width or floor is not positive and finite.
///
/// 返回给定峰值增益与半功率波束宽度时，天线在偏离视轴某角度处的增益。
///
/// 这是若干干扰评估所用的角平方模型，以半功率波束宽度表达，使任何天线
/// 都能由链路预算手头的两个数描述。以分贝计，
///
/// ```text
/// G(theta) = G_peak - min(12 (theta / theta_0)^2, G_floor)
/// ```
///
/// 其中 `theta_0` 是半功率波束宽度，`G_floor` 是所要降到的旁瓣电平，
/// 相对峰值以分贝计。角平方律来自均匀照射孔径 Airy 图案的首项，直到
/// 第一旁瓣都是良好近似。
///
/// 系数十二的选取使衰减在 `theta_0 / 2` 处达到三分贝，这正是半功率
/// 波束宽度的定义。在建议书适用之处，应改用取自建议书的图案：ITU-R
/// S.465 图案见 [`reference_pattern_gain`](crate::reference_pattern_gain)，
/// 它在主瓣上以及旁瓣区的形状上都与本模型不同。
///
/// 峰值增益没有分贝形式，或波束宽度、底限非正且非有限时返回 `None`。
pub fn off_axis_attenuation(
    peak: Gain,
    beam_width: f64,
    off_axis_angle: f64,
    floor_db: f64,
) -> Option<Decibel> {
    let peak_db = peak.as_decibel()?;
    if !beam_width.is_finite() || beam_width <= 0.0 {
        return None;
    }
    if !floor_db.is_finite() || floor_db <= 0.0 {
        return None;
    }
    if !off_axis_angle.is_finite() {
        return None;
    }
    let ratio = 12.0 * (off_axis_angle / beam_width).powi(2);
    let attenuation = ratio.min(floor_db);
    Some(peak_db - Decibel::from_db(attenuation))
}

/// Returns the gain that an antenna is required not to exceed at an angle
/// from boresight, in decibels relative to an isotropic radiator.
///
/// This is the reference pattern of the Radio Regulations Appendix 8, used
/// to check whether an Earth station meets the off-axis emission limits that
/// protect neighbouring satellites. The pattern is piecewise:
///
/// ```text
/// for  D/lambda >= 100:
///   G = 29 - 25 log10(theta)            for 1   <= theta < 36 degrees
///   G = -10                             for 36  <= theta <= 180 degrees
/// for  D/lambda <  100:
///   G = 32 - 25 log10(theta)            for 1   <= theta < 48 degrees
///   G = -10                             for 48  <= theta <= 180 degrees
///   G = 32 - 25 log10(48)               for theta < 1 degree
/// ```
///
/// The angle is in radians and the diameter to wavelength ratio decides
/// which branch applies, because a small antenna has a broader main lobe
/// and therefore a wider range over which the pattern must be controlled.
///
/// This function returns the limit, that is the largest gain permitted. An
/// antenna is compliant when its actual off-axis gain is at or below it.
///
/// Returns `None` when the angle is negative, when the ratio is not
/// positive, or when the angle is not finite.
///
/// Source: ITU, *Radio Regulations*, Appendix 8, which reproduces the
/// reference pattern of ITU-R Recommendation S.465.
///
/// 返回天线在偏离视轴某角度处不得超出的增益，相对各向同性辐射体以分贝
/// 计。
///
/// 这是《无线电规则》附录 8 的参考方向图，用于检查地球站是否满足保护
/// 邻近卫星的偏轴发射限值。该方向图是分段的：
///
/// ```text
/// for  D/lambda >= 100:
///   G = 29 - 25 log10(theta)            for 1   <= theta < 36 degrees
///   G = -10                             for 36  <= theta <= 180 degrees
/// for  D/lambda <  100:
///   G = 32 - 25 log10(theta)            for 1   <= theta < 48 degrees
///   G = -10                             for 48  <= theta <= 180 degrees
///   G = 32 - 25 log10(48)               for theta < 1 degree
/// ```
///
/// 角度以弧度计，口径与波长之比决定适用哪一支，因为小天线主瓣更宽，
/// 故须受控的角度范围也更大。
///
/// 本函数返回限值，即允许的最大增益。当实际偏轴增益不高于它时天线即
/// 合规。
///
/// 角度为负、比值为非正，或角度非有限时返回 `None`。
///
/// 出处：ITU，*Radio Regulations*，附录 8，其中再现了 ITU-R 建议书
/// S.465 的参考方向图。
pub fn mask_gain(diameter_over_wavelength: f64, off_axis_angle: f64) -> Option<f64> {
    if !diameter_over_wavelength.is_finite() || diameter_over_wavelength <= 0.0 {
        return None;
    }
    if !off_axis_angle.is_finite() || off_axis_angle < 0.0 {
        return None;
    }

    let large_aperture = diameter_over_wavelength >= 100.0;
    // The break point is where the pattern reaches its floor, and the cap
    // angle is where the falling branch begins.
    // 折点是方向图到达其底限之处，封顶角是下降支开始之处。
    let (cap_angle, floor_angle) = if large_aperture {
        (1.0_f64.to_radians(), 36.0_f64.to_radians())
    } else {
        (1.0_f64.to_radians(), 48.0_f64.to_radians())
    };
    let head = if large_aperture { 29.0 } else { 32.0 };

    if off_axis_angle < cap_angle {
        // Inside the first degree the pattern is taken at its value at one
        // degree, which is the most restrictive point on the falling branch.
        // 在一度以内，方向图取其一度的值，这是下降支上限制最严的点。
        Some(head - 25.0 * cap_angle.to_degrees().log10())
    } else if off_axis_angle <= floor_angle {
        Some(head - 25.0 * off_axis_angle.to_degrees().log10())
    } else {
        Some(-10.0)
    }
}

/// The shape of an antenna pattern.
///
/// A pattern is either described by its geometry, in which case the peak
/// gain and beam width are derived from the aperture, or given directly as
/// measured or published values. The enum lets a link budget accept either.
///
/// 天线方向图的形状。
///
/// 方向图或由其几何描述（此时峰值增益与波束宽度由孔径导出），或作为
/// 实测或已发表的值直接给出。该枚举使链路预算能接受二者。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pattern {
    /// A parabolic aperture, described by its diameter and efficiency.
    ///
    /// 抛物面孔径，由其直径与效率描述。
    Parabolic {
        /// The reflector diameter, in meters.
        ///
        /// 反射面直径，单位为米。
        diameter: f64,
        /// The aperture efficiency, dimensionless.
        ///
        /// 孔径效率，无量纲。
        efficiency: f64,
    },
    /// An antenna described directly by its peak gain and half power beam
    /// width.
    ///
    /// 直接由其峰值增益与半功率波束宽度描述的天线。
    Direct {
        /// The peak gain on boresight.
        ///
        /// 视轴上的峰值增益。
        ///
        /// 视轴上的峰值增益。
        peak: Gain,
        /// The half power beam width, in radians.
        ///
        /// 半功率波束宽度，单位为弧度。
        ///
        /// 半功率波束宽度，单位为弧度。
        beam_width: f64,
    },
}

impl Pattern {
    /// Returns the peak gain of this pattern at the given wavelength.
    ///
    /// The wavelength is ignored for a directly specified pattern, which
    /// already carries its peak gain.
    ///
    /// 返回本方向图在给定波长处的峰值增益。
    ///
    /// 对直接指定的方向图，波长被忽略，因为它已带有其峰值增益。
    pub fn peak_gain(self, wavelength: f64) -> Option<Gain> {
        match self {
            Self::Parabolic {
                diameter,
                efficiency,
            } => ParabolicAperture {
                diameter,
                efficiency,
            }
            .peak_gain(wavelength),
            Self::Direct { peak, .. } => Some(peak),
        }
    }

    /// Returns the half power beam width of this pattern at the given
    /// wavelength, in radians.
    ///
    /// 返回本方向图在给定波长处的半功率波束宽度，单位为弧度。
    pub fn beam_width(self, wavelength: f64) -> Option<f64> {
        match self {
            Self::Parabolic {
                diameter,
                efficiency,
            } => ParabolicAperture {
                diameter,
                efficiency,
            }
            .half_power_beam_width(wavelength),
            Self::Direct { beam_width, .. } => {
                (beam_width.is_finite() && beam_width > 0.0).then_some(beam_width)
            }
        }
    }
}

/// A gain pattern constructed from a peak gain and a half power beam width.
///
/// This is the form a link budget needs: the two numbers that describe the
/// beam, plus the model that gives the gain at any angle from boresight.
///
/// 由峰值增益与半功率波束宽度构造的增益方向图。
///
/// 这是链路预算所需的形式：描述波束的两个数，加上给出偏离视轴任意角度
/// 处增益的模型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElementPattern {
    /// The peak gain on boresight.
    pub peak: Gain,
    /// The half power beam width, in radians.
    pub beam_width: f64,
    /// The sidelobe floor, in decibels below the peak.
    ///
    /// 旁瓣底限，相对峰值以分贝计。
    pub floor_db: f64,
}

impl Default for ElementPattern {
    /// Returns a pattern for an isotropic antenna, with a floor that makes
    /// the attenuation zero in every direction.
    ///
    /// 返回各向同性天线的方向图，其底限使各方向的衰减为零。
    fn default() -> Self {
        Self {
            peak: Gain::ISOTROPIC,
            beam_width: PI,
            floor_db: f64::MIN_POSITIVE,
        }
    }
}

impl ElementPattern {
    /// Creates a pattern from a peak gain and a half power beam width.
    ///
    /// The sidelobe floor is set to the value of the Radio Regulations
    /// reference pattern, 32 decibels below the peak, which is a common
    /// worst case for a small Earth station.
    ///
    /// 由峰值增益与半功率波束宽度创建方向图。
    ///
    /// 旁瓣底限被设为《无线电规则》参考方向图的值，即相对峰值低
    /// 32 分贝，这是小型地球站常见的坏情形。
    pub const fn new(peak: Gain, beam_width: f64) -> Self {
        Self {
            peak,
            beam_width,
            floor_db: 32.0,
        }
    }

    /// Creates a pattern from a parabolic aperture at a wavelength.
    ///
    /// Returns `None` when the aperture or wavelength is not usable.
    ///
    /// 由某波长处的抛物面孔径创建方向图。
    ///
    /// 孔径或波长不可用时返回 `None`。
    pub fn from_parabolic(aperture: ParabolicAperture, wavelength: f64) -> Option<Self> {
        Some(Self {
            peak: aperture.peak_gain(wavelength)?,
            beam_width: aperture.half_power_beam_width(wavelength)?,
            floor_db: 32.0,
        })
    }

    /// Returns this pattern with a different sidelobe floor.
    ///
    /// 返回本方向图的一个具有不同旁瓣底限的副本。
    pub const fn with_floor(mut self, floor_db: f64) -> Self {
        self.floor_db = floor_db;
        self
    }

    /// Returns the gain at an angle from boresight.
    ///
    /// Returns `None` when the pattern is not usable or the angle is not
    /// finite.
    ///
    /// 返回偏离视轴某角度处的增益。
    ///
    /// 方向图不可用或角度非有限时返回 `None`。
    pub fn gain_at(self, off_axis_angle: f64) -> Option<Gain> {
        let attenuation =
            off_axis_attenuation(self.peak, self.beam_width, off_axis_angle, self.floor_db)?;
        Gain::from_dbi(attenuation.db())
    }

    /// Returns `true` when the beam width and floor describe a usable
    /// pattern.
    ///
    /// 当波束宽度与底限描述一个可用方向图时返回 `true`。
    pub fn is_valid(self) -> bool {
        self.peak.dbi().is_some()
            && self.beam_width.is_finite()
            && self.beam_width > 0.0
            && self.floor_db.is_finite()
            && self.floor_db > 0.0
    }
}
