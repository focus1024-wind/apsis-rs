//! The Earth Rotation Angle and the rotation it defines between the
//! inertial and Earth-fixed frames.

use apsis_math::Mat3;
use core::f64::consts::PI;

/// The Earth Rotation Angle at the J2000.0 epoch, in turns.
///
/// The epoch value is zero by the definition of the angle: the International
/// Astronomical Union defines the Earth Rotation Angle to be zero at
/// 2000-01-01T12:00:00 UT1.
///
/// J2000.0 历元处的地球自转角，以周计。
///
/// 按该角的定义，历元值就是零：国际天文学联合会把 2000-01-01T12:00:00 UT1
/// 处的地球自转角定义为零。
pub const ERA_J2000_TURNS: f64 = 0.0;

/// The rate of change of the Earth Rotation Angle, in turns per day of UT1.
///
/// This is `1.002 737 811 911 354 6`, which is `360.985 612 288 088`
/// degrees per day. It exceeds one turn per day because the Earth turns once
/// in a sidereal day, which is shorter than a solar day.
///
/// Source: IERS Conventions (2010), equation 8.32.
///
/// 地球自转角的变化率，以每 UT1 日的周数计。
///
/// 该值为 `1.002 737 811 911 354 6`，即每天 `360.985 612 288 088` 度。它
/// 超过每天一周，因为地球在一个恒星日内转一周，而恒星日短于太阳日。
///
/// 出处：IERS Conventions（2010），式 8.32。
pub const ERA_RATE_TURNS_PER_DAY: f64 = 1.002_737_811_911_354_6;

/// The number of days from the J2000.0 epoch to the given Julian Date on the
/// UT1 scale.
///
/// The J2000.0 epoch is Julian Date 2451545.0, which is noon on
/// 2000-01-01.
///
/// 从 J2000.0 历元到给定 UT1 时标儒略日的天数。
///
/// J2000.0 历元是儒略日 2451545.0，即 2000-01-01 的正午。
pub fn ut1_days_from_j2000(ut1_julian_date: f64) -> f64 {
    ut1_julian_date - 2_451_545.0
}

/// The Earth Rotation Angle at a given instant.
///
/// The angle is the rotation from the celestial intermediate origin to the
/// terrestrial intermediate origin, measured about the celestial
/// intermediate pole. It replaces Greenwich Mean Sidereal Time in modern
/// practice and, unlike sidereal time, is a linear function of UT1.
///
/// The value is held in turns. A turn is one full revolution, so the angle
/// is periodic with a period of one. The value returned is not reduced to
/// any interval, so differences between two angles are meaningful even when
/// the interval is longer than a day.
///
/// 某瞬时的地球自转角。
///
/// 该角是从天球中间原点到地球中间原点的转动，绕天球中间极量取。它在现代
/// 实践中取代格林尼治平恒星时，且与恒星时不同，它是 UT1 的线性函数。
///
/// 该值以周保存。一周即一整圈，故该角以 1 为周期。所返回的值不归约到任何
/// 区间，故即便区间长于一日，两角之差仍有意义。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct EarthRotationAngle {
    turns: f64,
}

impl EarthRotationAngle {
    /// Creates an angle from a value in turns.
    ///
    /// 由以周表示的值创建角。
    pub const fn from_turns(turns: f64) -> Self {
        Self { turns }
    }

    /// Creates an angle from a value in radians.
    ///
    /// 由以弧度表示的值创建角。
    pub fn from_radians(radians: f64) -> Self {
        Self {
            turns: radians / (2.0 * PI),
        }
    }

    /// Returns the angle for a Julian Date on the UT1 scale.
    ///
    /// 返回 UT1 时标上某儒略日对应的角。
    pub fn from_ut1_julian_date(ut1_julian_date: f64) -> Self {
        Self::from_ut1_days(ut1_days_from_j2000(ut1_julian_date))
    }

    /// Returns the angle for a number of days of UT1 since the J2000.0
    /// epoch.
    ///
    /// The expression is linear: `theta = 0.779 057 273 2640 + 1.002 737
    /// 909 350 795 * d`, where `d` is the interval in days. The constant
    /// term of `0.779 057` turns is the angle at the epoch itself, which is
    /// about 280.46 degrees.
    ///
    /// Source: IERS Conventions (2010), equation 8.32.
    ///
    /// 返回自 J2000.0 历元起的若干 UT1 天对应的角。
    ///
    /// 该式为线性：`theta = 0.779 057 273 2640 + 1.002 737 909 350 795 * d`，
    /// 其中 `d` 是以天计的间隔。`0.779 057` 周的常数项是历元本身的角，
    /// 约为 280.46 度。
    ///
    /// 出处：IERS Conventions（2010），式 8.32。
    pub fn from_ut1_days(days_since_j2000: f64) -> Self {
        const EPOCH_TURNS: f64 = 0.779_057_273_264_0;
        Self {
            turns: EPOCH_TURNS + ERA_RATE_TURNS_PER_DAY * days_since_j2000,
        }
    }

    /// Returns the angle in turns.
    ///
    /// The value is not reduced to `[0, 1)`.
    ///
    /// 返回以周表示的角。
    ///
    /// 该值不归约到 `[0, 1)`。
    pub const fn turns(self) -> f64 {
        self.turns
    }

    /// Returns the angle in turns, reduced to `[0, 1)`.
    ///
    /// 返回以周表示的角，归约到 `[0, 1)`。
    pub fn turns_positive(self) -> f64 {
        self.turns.rem_euclid(1.0)
    }

    /// Returns the angle in radians.
    ///
    /// The value is not reduced to `[0, 2 pi)`.
    ///
    /// 返回以弧度表示的角。
    ///
    /// 该值不归约到 `[0, 2 pi)`。
    pub fn radians(self) -> f64 {
        self.turns * 2.0 * PI
    }

    /// Returns the angle in radians, reduced to `[0, 2 pi)`.
    ///
    /// 返回以弧度表示的角，归约到 `[0, 2 pi)`。
    pub fn radians_positive(self) -> f64 {
        self.turns_positive() * 2.0 * PI
    }
}

/// Returns the rotation that maps a vector in the inertial frame to the
/// Earth-fixed frame.
///
/// The rotation is a clockwise turn about the third axis by the Earth
/// Rotation Angle. Applying it to an inertial vector yields the Earth-fixed
/// vector, so a vector that is fixed in inertial space acquires a negative
/// rotation as the Earth turns beneath it.
///
/// 返回把惯性系中的矢量映射到地固系的转动。
///
/// 该转动是绕第三轴按地球自转角作顺时针转。把它施加于惯性矢量即得地固
/// 矢量，故在惯性空间固定的矢量会随地球在其下方转动而获得一个负的
/// 转动。
pub fn rotation_eci_to_ecef(angle: EarthRotationAngle) -> Mat3 {
    let (sin, cos) = angle.radians_positive().sin_cos();
    Mat3::from_rows([cos, sin, 0.0], [-sin, cos, 0.0], [0.0, 0.0, 1.0])
}

/// Returns the rotation that maps a vector in the Earth-fixed frame to the
/// inertial frame.
///
/// This is the transpose of [`rotation_eci_to_ecef`], which for a rotation
/// matrix is also its inverse.
///
/// 返回把地固系中的矢量映射到惯性系的转动。
///
/// 它是 [`rotation_eci_to_ecef`] 的转置，对旋转矩阵而言也是其逆。
pub fn rotation_ecef_to_eci(angle: EarthRotationAngle) -> Mat3 {
    rotation_eci_to_ecef(angle).transpose()
}
