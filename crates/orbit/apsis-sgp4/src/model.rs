//! Gravity models used by the propagator.
//!
//! The propagator needs four numbers that describe the Earth: the
//! gravitational parameter, the equatorial radius, and the second and third
//! zonal harmonics. The SGP4 model was fitted to the WGS 72 values and the
//! standard element sets distributed by Space-Track are intended to be used
//! with them, so WGS 72 is the default.
//!
//! WGS 84 is offered because elements fitted against it also exist. Mixing
//! models with element sets fitted against the other model shifts the
//! position by kilometers, so the choice should match the source of the
//! elements.
//!
//! The derived values `xke`, `tumin`, and `j3oj2` follow from the four
//! defining numbers and are computed once at construction.
//!
//! Sources:
//!
//! - WGS 72 values as used by the SGP4 reference implementation in
//!   Vallado et al., *Revisiting Spacetrack Report #3*, AIAA 2006-6753,
//!   appendix. The values match the WGS 72 gravity field.
//! - WGS 84 values: NIMA TR8350.2, third edition, 2000.

/// A gravity model: the Earth constants used by the propagator.
///
/// 重力模型：传播器所用的地球常数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityModel {
    /// Gravitational parameter, in cubic kilometers per second squared.
    ///
    /// 引力参数，单位为立方千米每二次方秒。
    pub mu: f64,
    /// Equatorial radius, in kilometers.
    ///
    /// 赤道半径，单位为千米。
    pub radius: f64,
    /// Second zonal harmonic, dimensionless.
    ///
    /// 第二带谐，无量纲。
    pub j2: f64,
    /// Third zonal harmonic, dimensionless.
    ///
    /// 第三带谐，无量纲。
    pub j3: f64,
    /// Fourth zonal harmonic, dimensionless.
    ///
    /// 第四带谐，无量纲。
    pub j4: f64,
}

impl GravityModel {
    /// The WGS 72 gravity model.
    ///
    /// This is the model the SGP4 theory was fitted to and the one that
    /// matches the standard element sets.
    ///
    /// WGS 72 重力模型。
    ///
    /// 这是 SGP4 理论拟合所依据、且与标准根数集相符的模型。
    pub const WGS72: Self = Self {
        mu: 398_600.8,
        radius: 6_378.135,
        j2: 0.001_082_616,
        j3: -0.000_002_538_81,
        j4: -0.000_001_655_97,
    };

    /// The WGS 84 gravity model.
    ///
    /// Use this only with element sets fitted against WGS 84.
    ///
    /// WGS 84 重力模型。
    ///
    /// 仅对按 WGS 84 拟合的根数集使用。
    pub const WGS84: Self = Self {
        mu: 398_600.5,
        radius: 6_378.137,
        j2: 0.001_082_629_89,
        j3: -0.000_002_532_153_1,
        j4: -0.000_001_610_988_4,
    };

    /// Returns the reciprocal of the square root of the gravitational
    /// parameter, in seconds per kilometer to the three halves.
    ///
    /// 返回引力参数平方根的倒数，单位为秒每千米的三分之二次方。
    pub fn reciprocal_sqrt_mu(self) -> f64 {
        1.0 / self.mu.sqrt()
    }

    /// Returns the square root of the gravitational parameter, in kilometers
    /// to the three halves per second.
    ///
    /// 返回引力参数的平方根，单位为千米的三分之二次方每秒。
    pub fn sqrt_mu(self) -> f64 {
        self.mu.sqrt()
    }

    /// Returns `xke`, the Kepler constant, in units of `er^1.5 / min`.
    ///
    /// The propagator works in minutes and Earth radii, so the gravitational
    /// parameter is converted by
    ///
    /// ```text
    /// xke = 60 / sqrt(radius^3 / mu)
    /// ```
    ///
    /// which is the angular rate, in radians per minute, of a circular orbit
    /// of radius one Earth radius.
    ///
    /// 返回 `xke`（开普勒常数），单位为 `er^1.5 / min`。
    ///
    /// 传播器以分钟与地球半径为单位，故引力参数按下式换算：
    ///
    /// ```text
    /// xke = 60 / sqrt(radius^3 / mu)
    /// ```
    ///
    /// 它是一个地球半径圆轨道的角速率，单位为弧度每分钟。
    pub fn xke(self) -> f64 {
        60.0 / (self.radius * self.radius * self.radius / self.mu).sqrt()
    }

    /// Returns the reciprocal of `xke`, in minutes.
    ///
    /// This is the time unit of the model: the period of the reference
    /// circular orbit divided by two pi.
    ///
    /// 返回 `xke` 的倒数，单位为分钟。
    ///
    /// 它是模型的时间单位：参考圆轨道的周期除以二 pi。
    pub fn tumin(self) -> f64 {
        1.0 / self.xke()
    }

    /// Returns `j3 / j2`, which appears as a single ratio in the theory.
    ///
    /// 返回 `j3 / j2`，理论中以单一比值形式出现。
    pub fn j3_over_j2(self) -> f64 {
        self.j3 / self.j2
    }

    /// Returns `2 / 3`, the exponent the model uses in the mean motion and
    /// drag corrections.
    ///
    /// It appears as a power rather than a factor, in terms such as
    /// `(xke / mean_motion)^(2/3)`, so it is a constant and does not depend
    /// on this model.
    ///
    /// 返回 `2 / 3`，即模型在平运动与阻力修正中使用的指数。
    ///
    /// 它以幂而非因子出现，例如 `(xke / mean_motion)^(2/3)`，故为常数，与
    /// 本模型无关。
    pub fn x2o3(self) -> f64 {
        2.0 / 3.0
    }
}
