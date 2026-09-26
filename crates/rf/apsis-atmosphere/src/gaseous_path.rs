//! Slant path attenuation through the atmosphere.
//!
//! The attenuation along a slant path is not simply the specific attenuation
//! divided by the sine of the elevation, because the atmosphere thins with
//! height and the path is curved. ITU-R Recommendation P.676 reduces the
//! problem to two equivalent heights, one for dry air and one for water
//! vapour, so that the zenith attenuation is the specific attenuation at the
//! surface multiplied by that height.
//!
//! The equivalent height of dry air depends on frequency through the shape
//! of the oxygen complex, and the equivalent height of water vapour depends
//! on frequency and on the vapour density. Both depend on the surface
//! pressure and temperature.
//!
//! Source: ITU-R Recommendation P.676-12, equations 30 through 38.

use crate::gaseous::{Atmosphere, specific_attenuation_dry_air, specific_attenuation_water_vapour};

/// The coefficients of the dry air equivalent height, table 3 of P.676-12.
///
/// Each entry is the coefficient and the line frequency in gigahertz.
const DRY_HEIGHT_TERMS: &[(f64, f64)] = &[
    (0.159_7, 118.750_334),
    (0.106_6, 368.498_246),
    (0.132_5, 424.763_020),
    (0.124_2, 487.249_273),
    (0.093_8, 715.392_902),
    (0.144_8, 773.839_490),
    (0.137_4, 834.145_546),
];

/// The coefficients of the water vapour equivalent height, table 4 of
/// P.676-12.
///
/// Each entry is the line frequency in gigahertz, a coefficient, and a
/// width-like parameter.
const WET_HEIGHT_TERMS: &[(f64, f64, f64)] = &[
    (22.235_08, 1.52, 2.56),
    (183.310_087, 7.62, 10.2),
    (325.152_888, 1.56, 2.7),
    (380.197_353, 4.15, 5.7),
    (439.150_807, 0.2, 0.91),
    (448.001_085, 1.63, 2.46),
    (474.689_092, 0.76, 2.22),
    (488.490_108, 0.26, 2.49),
    (556.935_985, 7.81, 10.0),
    (620.700_87, 1.25, 2.35),
    (752.033_113, 16.2, 20.0),
    (916.171_582, 1.47, 2.58),
    (970.315_022, 1.36, 2.44),
    (987.926_764, 1.6, 1.86),
];

/// The equivalent height of dry air and of water vapour, in kilometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquivalentHeights {
    /// The equivalent height of dry air, in kilometers.
    pub dry_km: f64,
    /// The equivalent height of water vapour, in kilometers.
    pub wet_km: f64,
}

/// Returns the equivalent heights of dry air and water vapour.
///
/// The heights are the lengths that, multiplied by the surface specific
/// attenuation, give the zenith attenuation. The dry air height is limited
/// to a pressure dependent bound below 70 GHz, where the oxygen complex is
/// narrow enough that the height would otherwise be overestimated.
///
/// The frequency is in gigahertz.
///
/// Returns `None` when the frequency or atmosphere is not usable.
///
/// Source: ITU-R Recommendation P.676-12, equations 30 through 38.
pub fn equivalent_heights(frequency_ghz: f64, atmosphere: Atmosphere) -> Option<EquivalentHeights> {
    if !frequency_ghz.is_finite() || frequency_ghz <= 0.0 || !atmosphere.is_valid() {
        return None;
    }
    let temperature = atmosphere.temperature_k;
    let rho = atmosphere.water_vapour_density;
    let e = atmosphere.water_vapour_partial_pressure();
    let rp = (atmosphere.pressure_hpa + e) / 1_013.25;

    // Equation 31, the resonance term centred just below 60 GHz.
    let term1 = 5.104_0 / (1.0 + 0.066 * rp.powf(-2.3))
        * (-((frequency_ghz - 59.7) / (2.87 + 12.4 * (-7.9 * rp).exp())).powi(2)).exp();

    // Equation 32, the residual contribution of the submillimetre lines.
    let term2: f64 = DRY_HEIGHT_TERMS
        .iter()
        .map(|&(coefficient, line_frequency)| {
            coefficient * (2.12 * rp).exp()
                / ((frequency_ghz - line_frequency).powi(2) + 0.025 * (2.2 * rp).exp())
        })
        .sum();

    // Equation 33, the far wing correction.
    let term3 = 0.011_4 * frequency_ghz / (1.0 + 0.14 * rp.powf(-2.6))
        * (15.02 * frequency_ghz.powi(2) - 1_353.0 * frequency_ghz + 5.333e4)
        / (frequency_ghz.powi(3) - 151.3 * frequency_ghz.powi(2) + 9_629.0 * frequency_ghz
            - 6_803.0);

    let a = 0.783_2 + 0.007_09 * (temperature - 273.15);

    // Equation 30, with the pressure dependent cap that applies below
    // 70 GHz.
    let dry = 6.1 * a / (1.0 + 0.17 * rp.powf(-1.1)) * (1.0 + term1 + term2 + term3);
    let dry = if frequency_ghz < 70.0 {
        dry.min(10.7 * rp.powf(0.3))
    } else {
        dry
    };

    // Equations 36 through 38 and 35b.
    let a_wet = 1.929_8 - 0.041_66 * (temperature - 273.15) + 0.051_7 * rho;
    let b_wet = 1.167_4 - 0.006_22 * (temperature - 273.15) + 0.006_3 * rho;
    let sigma = 1.013 / (1.0 + (-8.6 * (rp - 0.57)).exp());

    let wet = a_wet
        + b_wet
            * WET_HEIGHT_TERMS
                .iter()
                .map(|&(line_frequency, coefficient, width)| {
                    coefficient * sigma / ((frequency_ghz - line_frequency).powi(2) + width * sigma)
                })
                .sum::<f64>();

    Some(EquivalentHeights {
        dry_km: dry,
        wet_km: wet,
    })
}

/// Returns the zenith attenuation of the atmosphere, in decibels.
///
/// This is the specific attenuation at the surface multiplied by the
/// equivalent height, summed over the two constituents.
///
/// The frequency is in gigahertz.
///
/// Returns `None` when the frequency or atmosphere is not usable.
pub fn zenith_attenuation(frequency_ghz: f64, atmosphere: Atmosphere) -> Option<f64> {
    let heights = equivalent_heights(frequency_ghz, atmosphere)?;
    let dry = specific_attenuation_dry_air(frequency_ghz, atmosphere)?;
    let wet = specific_attenuation_water_vapour(frequency_ghz, atmosphere)?;
    Some(dry * heights.dry_km + wet * heights.wet_km)
}

/// Returns the attenuation along a slant path at an elevation angle, in
/// decibels.
///
/// The zenith attenuation is divided by the sine of the elevation. The
/// plane parallel approximation this uses is the one the recommendation
/// applies for the approximate method; it is accurate above about five
/// degrees of elevation.
///
/// The elevation is in radians.
///
/// Returns `None` when the frequency or atmosphere is not usable, or when
/// the elevation is at or below the horizon.
pub fn slant_path_attenuation(
    frequency_ghz: f64,
    elevation: f64,
    atmosphere: Atmosphere,
) -> Option<f64> {
    if !elevation.is_finite() || elevation.sin() <= 0.0 {
        return None;
    }
    Some(zenith_attenuation(frequency_ghz, atmosphere)? / elevation.sin())
}
