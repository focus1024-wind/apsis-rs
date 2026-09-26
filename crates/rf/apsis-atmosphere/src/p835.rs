//! The reference standard atmosphere of ITU-R Recommendation P.835.
//!
//! The exact slant path method of Recommendation P.676 integrates the
//! specific attenuation through a stack of layers, so it needs the
//! temperature, pressure, and water vapour density at each layer height.
//! This module supplies those from the mean annual global reference
//! atmosphere of Recommendation P.835.
//!
//! Height is measured as geopotential height up to the base of the
//! thermosphere. The geopotential height is the geometric height adjusted
//! for the variation of gravity with altitude, and it is the independent
//! variable the temperature and pressure profiles are expressed in.
//!
//! Source: ITU-R Recommendation P.835-6, *Reference Standard Atmospheres*,
//! section 1, the mean annual global reference atmosphere.

/// The radius of the Earth used in the geopotential conversion, in
/// kilometers.
///
/// The recommendation fixes this value for the standard atmosphere, which is
/// slightly different from the polar radius used elsewhere. It is the value
/// that makes the geopotential height consistent with the tabulated
/// profiles.
const GEOPOTENTIAL_EARTH_RADIUS_KM: f64 = 6_356.766;

/// Returns the geopotential height for a geometric height, in kilometers.
///
/// The relation is
///
/// ```text
/// h_p = R H / (R + H)
/// ```
///
/// with `R` the fixed Earth radius of the standard atmosphere and `H` the
/// geometric height. The geopotential height is always the smaller of the
/// two.
pub fn geopotential_height(geometric_height_km: f64) -> f64 {
    GEOPOTENTIAL_EARTH_RADIUS_KM * geometric_height_km
        / (GEOPOTENTIAL_EARTH_RADIUS_KM + geometric_height_km)
}

/// Returns the temperature of the reference atmosphere, in kelvin.
///
/// The profile is piecewise linear in geopotential height up to 84.852 km
/// and is then defined by two further segments that extend to 100 km. Above
/// 100 km the recommendation does not define a global profile and this
/// function returns the value at 100 km.
///
/// The geometric height is in kilometers.
pub fn standard_temperature(geometric_height_km: f64) -> f64 {
    let height = geopotential_height(geometric_height_km);
    if height <= 11.0 {
        288.15 - 6.5 * height
    } else if height <= 20.0 {
        216.65
    } else if height <= 32.0 {
        216.65 + (height - 20.0)
    } else if height <= 47.0 {
        228.65 + 2.8 * (height - 32.0)
    } else if height <= 51.0 {
        270.65
    } else if height <= 71.0 {
        270.65 - 2.8 * (height - 51.0)
    } else if height <= 84.852 {
        214.65 - 2.0 * (height - 71.0)
    } else if (86.0..=91.0).contains(&geometric_height_km) {
        186.867_3
    } else if geometric_height_km > 91.0 && geometric_height_km <= 100.0 {
        let ratio = (geometric_height_km - 91.0) / 19.942_9;
        263.190_5 - 76.323_2 * (1.0 - ratio * ratio).max(0.0).sqrt()
    } else {
        195.081_34
    }
}

/// Returns the pressure of the reference atmosphere, in hectopascals.
///
/// The profile is piecewise, continuous at the segment boundaries, and
/// follows from the hydrostatic relation applied to each temperature
/// segment.
///
/// The geometric height is in kilometers.
pub fn standard_pressure(geometric_height_km: f64) -> f64 {
    let height = geopotential_height(geometric_height_km);
    if height <= 11.0 {
        1_013.25 * (288.15 / (288.15 - 6.5 * height)).powf(-34.163_2 / 6.5)
    } else if height <= 20.0 {
        226.322_6 * (-34.163_2 * (height - 11.0) / 216.65).exp()
    } else if height <= 32.0 {
        54.749_80 * (216.65 / (216.65 + (height - 20.0))).powf(34.163_2)
    } else if height <= 47.0 {
        8.680_422 * (228.65 / (228.65 + 2.8 * (height - 32.0))).powf(34.163_2 / 2.8)
    } else if height <= 51.0 {
        1.109_106 * (-34.163_2 * (height - 47.0) / 270.65).exp()
    } else if height <= 71.0 {
        0.669_416_7 * (270.65 / (270.65 - 2.8 * (height - 51.0))).powf(-34.163_2 / 2.8)
    } else if height <= 84.852 {
        0.039_566_49 * (214.65 / (214.65 - 2.0 * (height - 71.0))).powf(-34.163_2 / 2.0)
    } else if (86.0..=100.0).contains(&geometric_height_km) {
        let h = geometric_height_km;
        (95.571_899 - 4.011_801 * h + 6.424_731e-2 * h * h - 4.789_660e-4 * h * h * h
            + 1.340_543e-6 * h * h * h * h)
            .exp()
    } else {
        1.0e-62
    }
}

/// Returns the water vapour density of the reference atmosphere, in grams
/// per cubic meter.
///
/// The profile is exponential with a scale height of two kilometers, scaled
/// so that the density at the surface is the value given.
///
/// The geometric height is in kilometers.
pub fn standard_water_vapour_density(geometric_height_km: f64, surface_density: f64) -> f64 {
    surface_density * (-geometric_height_km / 2.0).exp()
}

/// Returns the radio refractive index of moist air, dimensionless.
///
/// The relation is
///
/// ```text
/// N = 77.6 P_d / T + 72 e / T + 3.75e5 e / T^2
/// n = 1 + N 1e-6
/// ```
///
/// with `P_d` the dry air pressure and `e` the water vapour partial
/// pressure, both in hectopascals, and `T` the temperature in kelvin. The
/// first two terms are the non-dispersive refractivity and the third is the
/// dipole term that gives water vapour its strong effect.
///
/// Source: ITU-R Recommendation P.453, which gives this expression for the
/// refractive index of moist air.
pub fn radio_refractive_index(
    dry_pressure_hpa: f64,
    vapour_pressure_hpa: f64,
    temperature_k: f64,
) -> f64 {
    if temperature_k == 0.0 {
        return 1.0;
    }
    let refractivity = 77.6 * dry_pressure_hpa / temperature_k
        + 72.0 * vapour_pressure_hpa / temperature_k
        + 3.75e5 * vapour_pressure_hpa / (temperature_k * temperature_k);
    1.0 + refractivity * 1.0e-6
}
