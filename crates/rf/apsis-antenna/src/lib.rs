//! Antenna radiation patterns and gain models.
//!
//! An antenna does not radiate equally in every direction. This crate models
//! how the gain varies with angle for the two shapes that appear in almost
//! every satellite link: a parabolic reflector, whose beam is a narrow
//! pencil, and an idealised element with a broad pattern.
//!
//! Three beam width conventions are used in practice and they are not
//! interchangeable:
//!
//! - The half power beam width is the angle between the two directions where
//!   the gain has fallen to half its peak, which is three decibels down.
//! - The null to null beam width is the angle between the first two zeros on
//!   either side of the main lobe. It is roughly twice the half power width
//!   for a uniformly illuminated aperture.
//! - The first sidelobe level is the peak gain outside the main lobe. For a
//!   uniformly illuminated circular aperture it is about minus 17.6 dB.
//!
//! Every function here takes or returns angles in radians and gains as
//! [`apsis_rf::Gain`], so a result can be fed straight into a link budget.
//!
//! Sources:
//!
//! - Parabolic aperture gain: `G = eta (pi D / lambda)^2`, where `eta` is
//!   the aperture efficiency, typically between 0.55 and 0.70 for a
//!   well adjusted dish.
//! - Half power beam width of a circular aperture: `theta = k lambda / D`
//!   with `k` about 1.02 for uniform illumination and close to 1.2 for
//!   typical illumination tapers.
//! - The co-polar reference pattern of a simple Earth station antenna, which
//!   falls as the square of the off-axis angle down to a sidelobe floor. This
//!   is the form used by ITU-R Recommendation S.465 to bound interference
//!   from one satellite system into another.
//!
//! ```
//! use apsis_antenna::ParabolicAperture;
//!
//! // A three meter dish at twelve gigahertz has a peak gain near 50 dBi.
//! let dish = ParabolicAperture {
//!     diameter: 3.0,
//!     efficiency: 0.65,
//! };
//! let peak = dish.peak_gain(0.025).expect("valid wavelength");
//! assert!(peak.dbi().expect("positive") > 45.0);
//! assert!(peak.dbi().expect("positive") < 50.0);
//! ```
//!
//! 天线辐射方向图与增益模型。
//!
//! 天线并非向各个方向均匀辐射。本 crate 为几乎每条卫星链路都会出现的两种
//! 形状建模增益随角度的变化：抛物面反射体，其波束是窄笔形；以及具有宽方向图
//! 的理想化单元。
//!
//! 实践中有三种波束宽度约定，它们不可互换：
//!
//! - 半功率波束宽度是增益降到峰值一半（即低三分贝）的两个方向之间的角。
//! - 零点到零点波束宽度是主瓣两侧头两个零点之间的角。对均匀照射孔径，它约
//!   为半功率宽度的两倍。
//! - 第一旁瓣电平是主瓣之外的峰值增益。对均匀照射的圆孔径，它约为负
//!   17.6 dB。
//!
//! 此处的每个函数都以弧度接收或返回角度，以 [`apsis_rf::Gain`] 接收或返回
//! 增益，故结果可直接送入链路预算。
//!
//! 出处：
//!
//! - 抛物面孔径增益：`G = eta (pi D / lambda)^2`，其中 `eta` 是孔径效率，
//!   对调准良好的天线通常在 0.55 与 0.70 之间。
//! - 圆孔径的半功率波束宽度：`theta = k lambda / D`，均匀照射时 `k` 约为
//!   1.02，典型照射锥削时接近 1.2。
//! - 简单地球站天线的同极化参考方向图，其随偏轴角的平方下降到旁瓣底限。
//!   这是 ITU-R 建议书 S.465 为限定从一个卫星系统到另一个的干扰所用的
//!   形式。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod parabolic;
mod pattern;
mod reference;

pub use parabolic::{ParabolicAperture, beam_width, floor_angle};
pub use pattern::{ElementPattern, Pattern, mask_gain, off_axis_attenuation};
pub use reference::{
    PEAK_GAIN_OFFSET_DB, diameter_over_wavelength_from_peak_gain, main_lobe_break_angle,
    reference_beam_width, reference_pattern_decibel, reference_pattern_gain, reference_peak_gain,
};
