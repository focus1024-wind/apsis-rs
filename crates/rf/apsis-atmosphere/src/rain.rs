//! Rain attenuation.
//!
//! The specific attenuation of rain is modelled as a power law in the rain
//! rate, `gamma = k R^alpha`, where `k` and `alpha` depend on the frequency,
//! on the polarisation, and on the elevation. The coefficients are given by
//! ITU-R Recommendation P.838 as closed form curve fits rather than as a
//! table, which is what the current revision of that recommendation uses.
//!
//! The attenuation along a slant path follows ITU-R Recommendation P.618.
//! The path length through the rain is limited by the height of the rain
//! layer above the station, and two reduction factors account for the fact
//! that the rain rate is not uniform along the path: a horizontal reduction
//! and a vertical adjustment. Both depend on the specific attenuation, so
//! the calculation is not a simple product of a rate and a length.
//!
//! Sources:
//!
//! - Specific attenuation coefficients: ITU-R Recommendation P.838-3,
//!   *Specific attenuation model for rain for use in prediction methods*,
//!   equations 2 and 3 and tables 1 and 2.
//! - Slant path method: ITU-R Recommendation P.618-13, *Propagation data and
//!   prediction methods required for the design of Earth-space
//!   telecommunication systems*, equations 1 through 10.

use apsis_rf::Decibel;

/// The effective radius of the Earth used by the path geometry, in
/// kilometers.
///
/// The value accounts for the refraction of the ray in the lower atmosphere,
/// which makes the Earth appear larger than it is.
///
/// 路径几何所用的地球有效半径，单位为千米。
///
/// 该值考虑了射线在低层大气中的折射，折射使地球显得比实际更大。
const EFFECTIVE_EARTH_RADIUS_KM: f64 = 8_500.0;

/// A mid latitude rain height, in kilometers above sea level.
///
/// Recommendation P.618 takes the rain height from the zero degree isotherm
/// height of Recommendation P.839, which is tabulated as a function of
/// latitude and longitude. Prefer [`rain_attenuation_at`], which looks the
/// value up, over [`rain_attenuation`] with this constant.
///
/// 中纬度雨顶高度，以海拔千米计。
///
/// 建议书 P.618 从建议书 P.839 的零度等温线高度取雨顶高度，后者按纬度
/// 与经度列表给出。相较用本常量调用 [`rain_attenuation`]，更宜用会查表
/// 的 [`rain_attenuation_at`]。
pub const DEFAULT_RAIN_HEIGHT_KM: f64 = 3.0;

/// One term of a P.838-3 curve fit.
///
/// P.838-3 曲线拟合的一项。
#[derive(Debug, Clone, Copy)]
struct CurveTerm {
    /// The amplitude coefficient.
    ///
    /// 幅度系数。
    amplitude: f64,
    /// The centre of the term, on a base ten logarithmic frequency axis.
    ///
    /// 该项的中心，在十进对数频率轴上。
    centre: f64,
    /// The width of the term, on a base ten logarithmic frequency axis.
    ///
    /// 该项的宽度，在十进对数频率轴上。
    width: f64,
}

/// The coefficients of the `k` curve fit for horizontal polarisation.
///
/// 水平极化 `k` 曲线拟合的系数。
const K_HORIZONTAL_TERMS: &[CurveTerm] = &[
    CurveTerm {
        amplitude: -5.339_80,
        centre: -0.100_08,
        width: 1.130_98,
    },
    CurveTerm {
        amplitude: -0.353_51,
        centre: 1.269_70,
        width: 0.454_00,
    },
    CurveTerm {
        amplitude: -0.237_89,
        centre: 0.860_36,
        width: 0.153_54,
    },
    CurveTerm {
        amplitude: -0.941_58,
        centre: 0.645_52,
        width: 0.168_17,
    },
];

/// The coefficients of the `k` curve fit for vertical polarisation.
///
/// 垂直极化 `k` 曲线拟合的系数。
const K_VERTICAL_TERMS: &[CurveTerm] = &[
    CurveTerm {
        amplitude: -3.805_95,
        centre: 0.569_34,
        width: 0.810_61,
    },
    CurveTerm {
        amplitude: -3.449_65,
        centre: -0.229_11,
        width: 0.510_59,
    },
    CurveTerm {
        amplitude: -0.399_02,
        centre: 0.730_42,
        width: 0.118_99,
    },
    CurveTerm {
        amplitude: 0.501_67,
        centre: 1.073_19,
        width: 0.271_95,
    },
];

/// The coefficients of the `alpha` curve fit for horizontal polarisation.
///
/// 水平极化 `alpha` 曲线拟合的系数。
const ALPHA_HORIZONTAL_TERMS: &[CurveTerm] = &[
    CurveTerm {
        amplitude: -0.143_18,
        centre: 1.824_42,
        width: -0.551_87,
    },
    CurveTerm {
        amplitude: 0.295_91,
        centre: 0.775_64,
        width: 0.198_22,
    },
    CurveTerm {
        amplitude: 0.321_77,
        centre: 0.637_73,
        width: 0.131_64,
    },
    CurveTerm {
        amplitude: -5.376_10,
        centre: -0.962_30,
        width: 1.478_28,
    },
    CurveTerm {
        amplitude: 16.172_1,
        centre: -3.299_80,
        width: 3.439_90,
    },
];

/// The coefficients of the `alpha` curve fit for vertical polarisation.
///
/// 垂直极化 `alpha` 曲线拟合的系数。
const ALPHA_VERTICAL_TERMS: &[CurveTerm] = &[
    CurveTerm {
        amplitude: -0.077_71,
        centre: 2.338_40,
        width: -0.762_84,
    },
    CurveTerm {
        amplitude: 0.567_27,
        centre: 0.955_45,
        width: 0.540_39,
    },
    CurveTerm {
        amplitude: -0.202_38,
        centre: 1.145_20,
        width: 0.268_09,
    },
    CurveTerm {
        amplitude: -48.299_1,
        centre: 0.791_669,
        width: 0.116_226,
    },
    CurveTerm {
        amplitude: 48.583_3,
        centre: 0.791_459,
        width: 0.116_479,
    },
];

/// The linear coefficients of the `k` curve fits.
///
/// `k` 曲线拟合的线性系数。
const K_SLOPE_HORIZONTAL: f64 = -0.189_61;
/// The constant of the horizontal `k` curve fit.
///
/// 水平 `k` 曲线拟合的常数。
const K_OFFSET_HORIZONTAL: f64 = 0.711_47;
/// The linear coefficient of the vertical `k` curve fit.
///
/// 垂直 `k` 曲线拟合的线性系数。
const K_SLOPE_VERTICAL: f64 = -0.163_98;
/// The constant of the vertical `k` curve fit.
///
/// 垂直 `k` 曲线拟合的常数。
const K_OFFSET_VERTICAL: f64 = 0.632_97;
/// The linear coefficient of the horizontal `alpha` curve fit.
///
/// 水平 `alpha` 曲线拟合的线性系数。
const ALPHA_SLOPE_HORIZONTAL: f64 = 0.678_49;
/// The constant of the horizontal `alpha` curve fit.
///
/// 水平 `alpha` 曲线拟合的常数。
const ALPHA_OFFSET_HORIZONTAL: f64 = -1.955_37;
/// The linear coefficient of the vertical `alpha` curve fit.
///
/// 垂直 `alpha` 曲线拟合的线性系数。
const ALPHA_SLOPE_VERTICAL: f64 = -0.053_739;
/// The constant of the vertical `alpha` curve fit.
///
/// 垂直 `alpha` 曲线拟合的常数。
const ALPHA_OFFSET_VERTICAL: f64 = 0.834_33;

/// A rain attenuation result at a given elevation.
///
/// 给定仰角下的降雨衰减结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RainAttenuation {
    /// The specific attenuation, in decibels per kilometer.
    ///
    /// 比衰减，单位为分贝每千米。
    pub specific_db_per_km: f64,
    /// The effective path length through the rain, in kilometers.
    ///
    /// 穿过降雨的有效路径长度，单位为千米。
    pub path_length_km: f64,
    /// The total attenuation along the path, in decibels.
    ///
    /// 沿路径的总衰减，单位为分贝。
    pub attenuation_db: f64,
    /// The horizontal reduction factor, dimensionless and at most one.
    ///
    /// 水平缩减因子，无量纲且至多为一。
    pub horizontal_reduction: f64,
    /// The vertical adjustment factor, dimensionless.
    ///
    /// The factor is the ratio of the effective path length to the longer
    /// of the two geometric path estimates. It is at most one when the path
    /// meets the rain layer at a steep angle and can exceed one at a shallow
    /// angle, which is the behaviour Recommendation P.618 gives.
    ///
    /// 垂直调整因子，无量纲。
    ///
    /// 该因子是有效路径长度与两个几何路径估计中较长者之比。当路径以陡
    /// 角与雨层相交时至多为一，以浅角相交时可超过一，这正是建议书
    /// P.618 所给出的行为。
    pub vertical_adjustment: f64,
}

impl RainAttenuation {
    /// Returns the total attenuation.
    ///
    /// 返回总衰减。
    pub fn attenuation(self) -> Decibel {
        Decibel::from_db(self.attenuation_db)
    }

    /// Returns `true` when every value is finite and not negative.
    ///
    /// 当每个值有限且非负时返回 `true`。
    pub fn is_valid(self) -> bool {
        self.specific_db_per_km.is_finite()
            && self.specific_db_per_km >= 0.0
            && self.path_length_km.is_finite()
            && self.path_length_km >= 0.0
            && self.attenuation_db.is_finite()
            && self.attenuation_db >= 0.0
    }
}

/// Evaluates one P.838-3 curve term.
///
/// The term is a Gaussian on a base ten logarithmic frequency axis, so the
/// frequency must be in gigahertz and positive.
///
/// 求一个 P.838-3 曲线项。
///
/// 该项是十进对数频率轴上的高斯，故频率必须以吉赫兹计且为正。
fn curve_term(term: CurveTerm, frequency_ghz: f64) -> f64 {
    let logarithm = frequency_ghz.log10();
    term.amplitude * (-((logarithm - term.centre) / term.width).powi(2)).exp()
}

/// Returns the `k` for a polarisation, before the elevation correction.
///
/// 返回某极化的 `k`，在仰角修正之前。
fn base_k(terms: &[CurveTerm], slope: f64, offset: f64, frequency_ghz: f64) -> f64 {
    let gaussian: f64 = terms
        .iter()
        .map(|term| curve_term(*term, frequency_ghz))
        .sum();
    10f64.powf(gaussian + slope * frequency_ghz.log10() + offset)
}

/// Returns the `alpha` for a polarisation, before the elevation correction.
///
/// 返回某极化的 `alpha`，在仰角修正之前。
fn base_alpha(terms: &[CurveTerm], slope: f64, offset: f64, frequency_ghz: f64) -> f64 {
    let gaussian: f64 = terms
        .iter()
        .map(|term| curve_term(*term, frequency_ghz))
        .sum();
    gaussian + slope * frequency_ghz.log10() + offset
}

/// Returns the P.838-3 coefficients of the specific attenuation power law.
///
/// The polarisation is an angle in radians between the electric field and
/// the horizontal, so zero is horizontal and a right angle is vertical. The
/// elevation is in radians.
///
/// The result is a pair of the coefficient `k` and the exponent `alpha` of
/// the relation `gamma = k R^alpha`, with the rain rate in millimeters per
/// hour and the result in decibels per kilometer.
///
/// Returns `None` when the frequency, polarisation, or elevation is not
/// usable. The recommendation is stated for frequencies from 1 GHz to
/// 1000 GHz.
///
/// Source: ITU-R Recommendation P.838-3, equations 2 and 3.
///
/// 返回比衰减幂律的 P.838-3 系数。
///
/// 极化是电场与水平方向之间以弧度计的角度，故零为水平、直角为垂直。
/// 仰角以弧度计。
///
/// 结果是关系 `gamma = k R^alpha` 中系数 `k` 与指数 `alpha` 的一对值，
/// 其中降雨率以毫米每小时计，结果以分贝每千米计。
///
/// 频率、极化或仰角不可用时返回 `None`。该建议书针对 1 GHz 至 1000 GHz
/// 的频率陈述。
///
/// 出处：ITU-R 建议书 P.838-3，式 2 与式 3。
pub fn rain_coefficients(frequency: f64, polarisation: f64, elevation: f64) -> Option<(f64, f64)> {
    let frequency_ghz = frequency / 1.0e9;
    if !frequency_ghz.is_finite() || !(1.0..=1_000.0).contains(&frequency_ghz) {
        return None;
    }
    if !polarisation.is_finite() || !elevation.is_finite() {
        return None;
    }

    let k_horizontal = base_k(
        K_HORIZONTAL_TERMS,
        K_SLOPE_HORIZONTAL,
        K_OFFSET_HORIZONTAL,
        frequency_ghz,
    );
    let k_vertical = base_k(
        K_VERTICAL_TERMS,
        K_SLOPE_VERTICAL,
        K_OFFSET_VERTICAL,
        frequency_ghz,
    );
    let alpha_horizontal = base_alpha(
        ALPHA_HORIZONTAL_TERMS,
        ALPHA_SLOPE_HORIZONTAL,
        ALPHA_OFFSET_HORIZONTAL,
        frequency_ghz,
    );
    let alpha_vertical = base_alpha(
        ALPHA_VERTICAL_TERMS,
        ALPHA_SLOPE_VERTICAL,
        ALPHA_OFFSET_VERTICAL,
        frequency_ghz,
    );

    // The elevation and polarisation enter through the square of the cosine
    // of the elevation angle and the cosine of twice the polarisation tilt.
    // The elevation term vanishes at the zenith, where the distinction
    // between horizontal and vertical polarisation disappears.
    // 仰角与极化通过仰角余弦的平方以及二倍极化倾角的余弦进入。仰角项在
    // 天顶处消失，那里水平与垂直极化之别不复存在。
    let geometry_factor = elevation.cos().powi(2) * (2.0 * polarisation).cos();
    let k = 0.5 * (k_horizontal + k_vertical + (k_horizontal - k_vertical) * geometry_factor);
    let alpha = (k_horizontal * alpha_horizontal
        + k_vertical * alpha_vertical
        + (k_horizontal * alpha_horizontal - k_vertical * alpha_vertical) * geometry_factor)
        / (2.0 * k);
    Some((k, alpha))
}

/// Returns the specific attenuation in rain, in decibels per kilometer.
///
/// The rain rate is in millimeters per hour.
///
/// Returns `None` when the rain rate is negative, or when the coefficients
/// are not available.
///
/// 返回雨中的比衰减，单位为分贝每千米。
///
/// 降雨率以毫米每小时计。
///
/// 降雨率为负，或系数不可用时返回 `None`。
pub fn specific_rain_attenuation(
    rain_rate: f64,
    frequency: f64,
    polarisation: f64,
    elevation: f64,
) -> Option<f64> {
    if !rain_rate.is_finite() || rain_rate < 0.0 {
        return None;
    }
    let (k, alpha) = rain_coefficients(frequency, polarisation, elevation)?;
    Some(k * rain_rate.powf(alpha))
}

/// Returns the total rain attenuation along a slant path.
///
/// The rain rate is the value exceeded for the percentage of time of
/// interest, in millimeters per hour. The station height above sea level and
/// the rain layer height are in kilometers; pass [`DEFAULT_RAIN_HEIGHT_KM`]
/// for the latter when no local value is available.
///
/// The method is the one of Recommendation P.618. The slant path length
/// follows from the rain height, the station height, and the elevation, with
/// a correction for the curvature of the Earth. Two reduction factors then
/// account for the non-uniformity of the rain: a horizontal factor that
/// depends on the specific attenuation, the frequency, and the horizontal
/// projection of the path, and a vertical factor that depends on the
/// latitude through the angle at which the path crosses the rain layer.
///
/// Returns `None` when the rain rate, frequency, or elevation is not usable,
/// when the station is above the rain layer, or when the elevation is at or
/// below the horizon.
///
/// Source: ITU-R Recommendation P.618-13, equations 1 through 10.
///
/// 返回沿斜路径的总降雨衰减。
///
/// 降雨率是在所关注时间百分比上被超过的值，以毫米每小时计。站的海拔
/// 高度与雨层高度以千米计；没有本地值时，后者传
/// [`DEFAULT_RAIN_HEIGHT_KM`]。
///
/// 方法即建议书 P.618 所述。斜路径长度由雨顶高度、站高与仰角得出，
/// 并作地球曲率修正。随后两个缩减因子计入降雨的不均匀性：一个水平
/// 因子，取决于比衰减、频率以及路径的水平投影；一个垂直因子，通过
/// 路径穿越雨层的角度取决于纬度。
///
/// 降雨率、频率或仰角不可用，站高于雨层，或仰角在地平及其以下时返回
/// `None`。
///
/// 出处：ITU-R 建议书 P.618-13，式 1 至式 10。
pub fn rain_attenuation(
    rain_rate: f64,
    frequency: f64,
    polarisation: f64,
    elevation: f64,
    station_height_km: f64,
    rain_height_km: f64,
    latitude: f64,
) -> Option<RainAttenuation> {
    if !rain_rate.is_finite() || rain_rate < 0.0 {
        return None;
    }
    if !elevation.is_finite() || elevation.sin() <= 0.0 {
        return None;
    }
    if !station_height_km.is_finite() || !rain_height_km.is_finite() {
        return None;
    }
    if !latitude.is_finite() {
        return None;
    }
    let specific = specific_rain_attenuation(rain_rate, frequency, polarisation, elevation)?;

    // Equation 2: the slant path length, with the Earth curvature term.
    // 式 2：斜路径长度，含地球曲率项。
    let height_difference = rain_height_km - station_height_km;
    if height_difference <= 0.0 {
        // The station is at or above the rain layer, so the path is empty.
        // 站在雨层处或其上，故路径为空。
        return Some(RainAttenuation {
            specific_db_per_km: specific,
            path_length_km: 0.0,
            attenuation_db: 0.0,
            horizontal_reduction: 1.0,
            vertical_adjustment: 1.0,
        });
    }
    let sine = elevation.sin();
    let slant_length = 2.0 * height_difference
        / ((sine * sine + 2.0 * height_difference / EFFECTIVE_EARTH_RADIUS_KM).sqrt() + sine);

    // Equation 3: the horizontal projection of the slant path.
    // 式 3：斜路径的水平投影。
    let horizontal_length = (slant_length * elevation.cos()).abs();

    // Equation 6: the horizontal reduction factor.
    // 式 6：水平缩减因子。
    let frequency_ghz = frequency / 1.0e9;
    let horizontal_reduction = 1.0
        / (1.0 + 0.78 * (horizontal_length * specific / frequency_ghz).sqrt()
            - 0.38 * (1.0 - (-2.0 * horizontal_length).exp()));

    // Equation 7: the angle at which the path crosses the rain layer.
    // 式 7：路径穿越雨层的角度。
    let eta = (height_difference / (horizontal_length * horizontal_reduction)).atan();

    // Equation 8: the longer of the two path estimates.
    // 式 8：两个路径估计中较长者。
    let path_length = if eta > elevation {
        horizontal_length * horizontal_reduction / elevation.cos()
    } else {
        height_difference / sine
    };

    // Equation 9: the vertical adjustment factor.
    // 式 9：垂直调整因子。
    //
    // The exponential takes the elevation in degrees, not radians, because
    // the coefficients of 31 and 0.45 were fitted with degrees. The sine
    // takes the elevation in radians, as usual.
    // 指数以度而非弧度接收仰角，因为 31 与 0.45 两个系数是以度拟合的。
    // 正弦照常以弧度接收仰角。
    let latitude_magnitude = latitude.abs();
    let xi = if latitude_magnitude < 36.0 {
        36.0 - latitude_magnitude
    } else {
        0.0
    };
    let elevation_degrees = elevation.to_degrees();
    let vertical_adjustment = 1.0
        / (1.0
            + sine.sqrt()
                * (31.0
                    * (1.0 - (-(elevation_degrees / (1.0 + xi))).exp())
                    * (path_length * specific).sqrt()
                    / frequency_ghz.powi(2)
                    - 0.45));

    // Equation 10: the effective path length and the attenuation.
    // 式 10：有效路径长度与衰减。
    let effective_length = path_length * vertical_adjustment;
    Some(RainAttenuation {
        specific_db_per_km: specific,
        path_length_km: effective_length,
        attenuation_db: specific * effective_length,
        horizontal_reduction,
        vertical_adjustment,
    })
}

/// Returns the angle in radians for a polarisation described by a tilt from
/// horizontal.
///
/// This is a convenience for the two cases that are quoted by name.
///
/// 返回以相对水平倾角描述的极化所对应的弧度角。
///
/// 这是为两种常被具名引用的情形提供的便利。
pub const fn polarisation_tilt(degrees_from_horizontal: f64) -> f64 {
    degrees_from_horizontal * core::f64::consts::PI / 180.0
}

/// Returns the total rain attenuation along a slant path, taking the rain
/// height from the Recommendation P.839 model.
///
/// This is the same calculation as [`rain_attenuation`], with the rain layer
/// height looked up from the tabulated zero degree isotherm height at the
/// given latitude and longitude rather than supplied by the caller. It is
/// the form to use unless a locally measured isotherm height is available.
///
/// The station height must be given; pass zero for a station at sea level.
///
/// Returns `None` when the coordinates are not usable, or when any of the
/// other arguments is rejected by [`rain_attenuation`].
///
/// Source: the rain height is from ITU-R Recommendation P.839-4 and the path
/// method from ITU-R Recommendation P.618-13.
///
/// 返回沿斜路径的总降雨衰减，雨顶高度取自建议书 P.839 模型。
///
/// 这与 [`rain_attenuation`] 是同一计算，只是雨层高度由给定纬经度处列表
/// 给出的零度等温线高度查得，而非由调用方提供。除非有本地实测的等温线
/// 高度，否则宜用此形式。
///
/// 站高必须给出；海拔为零的站传零。
///
/// 坐标不可用，或其他任何参数被 [`rain_attenuation`] 拒绝时返回 `None`。
///
/// 出处：雨顶高度来自 ITU-R 建议书 P.839-4，路径方法来自 ITU-R 建议书
/// P.618-13。
pub fn rain_attenuation_at(
    rain_rate: f64,
    frequency: f64,
    polarisation: f64,
    elevation: f64,
    station_height_km: f64,
    latitude: f64,
    longitude: f64,
) -> Option<RainAttenuation> {
    let rain_height_km = crate::p839::rain_height(latitude, longitude)?;
    rain_attenuation(
        rain_rate,
        frequency,
        polarisation,
        elevation,
        station_height_km,
        rain_height_km,
        latitude,
    )
}
