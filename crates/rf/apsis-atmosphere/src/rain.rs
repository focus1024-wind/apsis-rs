//! Rain attenuation.

use crate::geometry::air_mass;
use apsis_rf::Decibel;
use core::f64::consts::PI;

/// A rain attenuation result at a given elevation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RainAttenuation {
    /// The specific attenuation, in decibels per kilometer.
    pub specific_db_per_km: f64,
    /// The effective path length through the rain, in kilometers.
    pub path_length_km: f64,
    /// The total attenuation along the path, in decibels.
    pub attenuation_db: f64,
}

impl RainAttenuation {
    /// Returns the total attenuation.
    pub fn attenuation(self) -> Decibel {
        Decibel::from_db(self.attenuation_db)
    }

    /// Returns `true` when every value is finite and not negative.
    pub fn is_valid(self) -> bool {
        self.specific_db_per_km.is_finite()
            && self.specific_db_per_km >= 0.0
            && self.path_length_km.is_finite()
            && self.path_length_km >= 0.0
            && self.attenuation_db.is_finite()
            && self.attenuation_db >= 0.0
    }
}

/// Returns the coefficients of the specific attenuation power law for a
/// frequency.
///
/// The specific attenuation in rain is modelled as
///
/// ```text
/// gamma = k R^alpha
/// ```
///
/// where `R` is the rain rate in millimeters per hour and `k` and `alpha`
/// depend on frequency, on the polarisation, and weakly on the elevation and
/// the rain temperature. The values returned here are for a frequency in
/// gigahertz, interpolated logarithmically in `k` and linearly in `alpha`
/// between the published points.
///
/// The polarisation is given as an angle in radians between the electric
/// field and the horizontal, so zero is horizontal, a right angle is
/// vertical, and the intermediate values are circular or slant
/// polarisation. The coefficients at an angle follow from the horizontal and
/// vertical values through
///
/// ```text
/// k     = (k_h + k_v + (k_h - k_v) cos^2(angle) cos(2 elevation)) / 2
/// alpha = (k_h a_h + k_v a_v + (k_h a_h - k_v a_v) cos^2(angle) cos(2 elevation)) / (2 k)
/// ```
///
/// Returns `None` when the frequency is outside the range the tables cover,
/// from 1 GHz to 1000 GHz.
///
/// Source: ITU-R Recommendation P.838, *Specific attenuation model for rain
/// for use in prediction methods*, which tabulates `k` and `alpha` for
/// horizontal and vertical polarisation over this frequency range.
pub fn rain_coefficients(frequency: f64, polarisation: f64, elevation: f64) -> Option<(f64, f64)> {
    if !frequency.is_finite() || !(1.0..=1_000.0).contains(&(frequency / 1.0e9)) {
        return None;
    }
    if !polarisation.is_finite() || !elevation.is_finite() {
        return None;
    }
    let gigahertz = frequency / 1.0e9;
    let (horizontal, vertical) = interpolate_coefficients(gigahertz);
    let cos_squared = polarisation.cos().powi(2);
    let cos_two_elevation = (2.0 * elevation).cos();

    let k = 0.5
        * (horizontal.0
            + vertical.0
            + (horizontal.0 - vertical.0) * cos_squared * cos_two_elevation);
    let numerator = horizontal.0 * horizontal.1
        + vertical.0 * vertical.1
        + (horizontal.0 * horizontal.1 - vertical.0 * vertical.1) * cos_squared * cos_two_elevation;
    let alpha = if k > 0.0 { numerator / (2.0 * k) } else { 1.0 };
    Some((k, alpha))
}

/// Returns the specific attenuation in rain, in decibels per kilometer.
///
/// The rain rate is in millimeters per hour. Returns `None` when the rain
/// rate is negative, or when the coefficients are not available.
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
/// The path is modelled as a rain layer of fixed height, so the geometric
/// path length through it is the layer height divided by the sine of the
/// elevation. A reduction factor is applied because the rain rate is not
/// uniform along the path and the longest paths are the ones where the
/// assumption of uniformity fails most.
///
/// The reduction factor used here is
///
/// ```text
/// r = 1 / (1 + L_G / L_0)
/// ```
///
/// with `L_G` the geometric path and `L_0` a reference length that depends
/// on the rain rate, which is the form given by ITU-R Recommendation P.618.
/// The height of the rain layer is taken as 3 km, a mid latitude value; a
/// caller working in the tropics should raise it and one in polar regions
/// should lower it.
///
/// Returns `None` when the rain rate, frequency, or elevation is not usable,
/// or when the elevation is at or below the horizon.
///
/// Source: ITU-R Recommendation P.618, *Propagation data and prediction
/// methods required for the design of Earth-space telecommunication
/// systems*.
pub fn rain_attenuation(
    rain_rate: f64,
    frequency: f64,
    polarisation: f64,
    elevation: f64,
) -> Option<RainAttenuation> {
    if !rain_rate.is_finite() || rain_rate < 0.0 {
        return None;
    }
    if !elevation.is_finite() || elevation.sin() <= 0.0 {
        return None;
    }
    let specific = specific_rain_attenuation(rain_rate, frequency, polarisation, elevation)?;

    // The geometric path through the rain layer, and the reduction factor
    // that accounts for its non-uniformity.
    const RAIN_HEIGHT_KM: f64 = 3.0;
    let geometric_path_km = RAIN_HEIGHT_KM * air_mass(elevation);
    let reference_length_km = 35.0 * (-0.015 * rain_rate).exp();
    let reduction = 1.0 / (1.0 + geometric_path_km / reference_length_km);
    let effective_path_km = geometric_path_km * reduction;

    Some(RainAttenuation {
        specific_db_per_km: specific,
        path_length_km: effective_path_km,
        attenuation_db: specific * effective_path_km,
    })
}

/// The published `k` and `alpha` values at the tabulated frequencies.
///
/// The table lists the horizontal value first and the vertical second, each
/// as a `k` and an `alpha`. The frequencies are the ones ITU-R
/// Recommendation P.838 tabulates.
/// One table entry: the frequency in gigahertz, then the horizontal and
/// vertical `(k, alpha)` pairs.
type CoefficientEntry = (f64, (f64, f64), (f64, f64));

const COEFFICIENT_TABLE: &[CoefficientEntry] = &[
    (1.0, (0.000_025_9, 0.969_1), (0.000_030_8, 0.859_2)),
    (2.0, (0.000_124, 0.963_0), (0.000_138, 0.948_5)),
    (4.0, (0.000_525, 1.075_1), (0.000_591, 1.075_1)),
    (6.0, (0.001_55, 1.265_6), (0.001_55, 1.312_4)),
    (8.0, (0.003_95, 1.368_2), (0.003_95, 1.327_3)),
    (10.0, (0.008_87, 1.264_3), (0.008_87, 1.121_6)),
    (12.0, (0.016_8, 1.200_0), (0.016_8, 1.000_0)),
    (15.0, (0.036_7, 1.154_0), (0.033_5, 1.000_0)),
    (20.0, (0.075_1, 1.099_0), (0.069_1, 1.000_0)),
    (25.0, (0.124, 1.061_0), (0.113, 1.030_0)),
    (30.0, (0.187, 1.021_0), (0.167, 1.000_0)),
    (35.0, (0.263, 0.979_0), (0.233, 0.963_0)),
    (40.0, (0.350, 0.939_0), (0.310, 0.929_0)),
    (45.0, (0.442, 0.903_0), (0.393, 0.897_0)),
    (50.0, (0.536, 0.873_0), (0.479, 0.868_0)),
    (60.0, (0.707, 0.826_0), (0.642, 0.824_0)),
    (70.0, (0.851, 0.793_0), (0.784, 0.793_0)),
    (80.0, (0.975, 0.769_0), (0.906, 0.769_0)),
    (90.0, (1.06, 0.753_0), (0.999, 0.754_0)),
    (100.0, (1.12, 0.743_0), (1.06, 0.744_0)),
    (120.0, (1.18, 0.731_0), (1.13, 0.732_0)),
    (150.0, (1.31, 0.710_0), (1.27, 0.711_0)),
    (200.0, (1.45, 0.689_0), (1.42, 0.690_0)),
    (300.0, (1.36, 0.688_0), (1.35, 0.689_0)),
    (400.0, (1.32, 0.683_0), (1.31, 0.684_0)),
    (500.0, (1.26, 0.664_0), (1.25, 0.664_0)),
    (600.0, (1.21, 0.650_0), (1.20, 0.650_0)),
    (700.0, (1.17, 0.637_0), (1.16, 0.638_0)),
    (800.0, (1.13, 0.626_0), (1.12, 0.627_0)),
    (900.0, (1.09, 0.618_0), (1.09, 0.619_0)),
    (1_000.0, (1.06, 0.610_0), (1.06, 0.611_0)),
];

/// Returns the horizontal and vertical coefficients at a frequency, by
/// interpolation of the table.
///
/// The interpolation is logarithmic in `k` and linear in `alpha`, which is
/// the convention the recommendation uses, because `k` spans several orders
/// of magnitude while `alpha` is close to one throughout.
fn interpolate_coefficients(gigahertz: f64) -> ((f64, f64), (f64, f64)) {
    let table = COEFFICIENT_TABLE;
    // The table is ascending in frequency and its range is checked by the
    // caller, so the bracketing pair always exists.
    let upper = table
        .iter()
        .position(|entry| entry.0 >= gigahertz)
        .unwrap_or(table.len() - 1);
    let lower = upper.saturating_sub(1);
    let (low_frequency, low_horizontal, low_vertical) = table[lower];
    let (high_frequency, high_horizontal, high_vertical) = table[upper];
    if high_frequency == low_frequency {
        return (low_horizontal, low_vertical);
    }
    let fraction = (gigahertz - low_frequency) / (high_frequency - low_frequency);

    let interpolate = |low: (f64, f64), high: (f64, f64)| {
        let k = low.0 * (high.0 / low.0).powf(fraction);
        let alpha = low.1 + (high.1 - low.1) * fraction;
        (k, alpha)
    };
    (
        interpolate(low_horizontal, high_horizontal),
        interpolate(low_vertical, high_vertical),
    )
}

/// Returns the angle in radians for a polarisation described by a tilt from
/// horizontal.
///
/// This is a convenience for the two cases that are quoted by name.
pub const fn polarisation_tilt(degrees_from_horizontal: f64) -> f64 {
    degrees_from_horizontal * PI / 180.0
}
