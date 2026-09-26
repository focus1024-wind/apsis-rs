//! Gaseous attenuation.

use crate::geometry::air_mass;
use apsis_rf::Decibel;

/// The zenith attenuation of a gaseous absorber, split into its two parts.
///
/// Oxygen and water vapour are carried separately because they respond very
/// differently to frequency and to the weather: the water vapour term
/// depends on the humidity and the oxygen term does not.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GaseousAttenuation {
    /// The oxygen contribution at the zenith, in decibels.
    pub oxygen_db: f64,
    /// The water vapour contribution at the zenith, in decibels.
    pub water_vapour_db: f64,
}

impl GaseousAttenuation {
    /// Creates a gaseous attenuation from its two zenith components.
    pub const fn new(oxygen_db: f64, water_vapour_db: f64) -> Self {
        Self {
            oxygen_db,
            water_vapour_db,
        }
    }

    /// Returns the total zenith attenuation, in decibels.
    pub fn zenith_db(self) -> f64 {
        self.oxygen_db + self.water_vapour_db
    }

    /// Returns the total attenuation at an elevation angle, in decibels.
    ///
    /// The zenith value is scaled by the air mass of the slant path. The
    /// result is infinite at or below the horizon.
    pub fn at_elevation(self, elevation: f64) -> Decibel {
        Decibel::from_db(self.zenith_db() * air_mass(elevation))
    }

    /// Returns `true` when both components are finite and not negative.
    pub fn is_valid(self) -> bool {
        self.oxygen_db.is_finite()
            && self.oxygen_db >= 0.0
            && self.water_vapour_db.is_finite()
            && self.water_vapour_db >= 0.0
    }
}

/// Returns the zenith attenuation of oxygen at a frequency, in decibels.
///
/// The model is a gross simplification of the oxygen absorption complex,
/// which is a set of lines near 60 GHz and a single line near 118 GHz. It
/// captures the shape that matters for a link budget: negligible attenuation
/// at the common satellite bands below 20 GHz, a rapid rise into the 60 GHz
/// complex, and a fall on the far side of it.
///
/// The value is the attenuation through the whole atmosphere at the zenith,
/// around 4 dB in the 60 GHz complex. Outside the strong absorption region
/// it is a small fraction of a decibel.
///
/// Returns `None` when the frequency is not positive and finite.
///
/// Source: the shape follows ITU-R Recommendation P.676, *Attenuation by
/// atmospheric gases*, which tabulates the line by line result this function
/// approximates.
pub fn oxygen_zenith_attenuation(frequency: f64) -> Option<f64> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return None;
    }
    let gigahertz = frequency / 1.0e9;
    // A narrow complex centred near 60 GHz, plus a weak line at 118 GHz.
    let complex = gaussian(gigahertz, 60.0, 8.0, 15.0);
    let line = gaussian(gigahertz, 118.0, 12.0, 2.0);
    // A baseline that grows slowly with frequency from the far wings of the
    // lines and from non-resonant absorption.
    let baseline = 0.006 * gigahertz.powf(1.5).min(1.5);
    Some(complex + line + baseline)
}

/// Returns the zenith attenuation of water vapour at a frequency, in
/// decibels.
///
/// Water vapour has a single absorption line at about 22.235 GHz, which is
/// the dominant feature for satellite links, and a much weaker set of lines
/// above 100 GHz. This function models the 22 GHz line in the shape that
/// ITU-R Recommendation P.676 tabulates for a standard atmosphere.
///
/// The value is the zenith attenuation through the whole atmosphere, about
/// 0.2 dB at the line for a typical water vapour column.
///
/// Returns `None` when the frequency is not positive and finite.
///
/// Source: shape from ITU-R Recommendation P.676.
pub fn water_vapour_zenith_attenuation(frequency: f64) -> Option<f64> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return None;
    }
    let gigahertz = frequency / 1.0e9;
    // The 22 GHz line, with a much weaker set of lines above 180 GHz.
    let line = gaussian(gigahertz, 22.235, 4.0, 0.20);
    let upper = gaussian(gigahertz, 183.0, 20.0, 4.0);
    Some(line + upper)
}

/// Returns the total gaseous zenith attenuation at a frequency, in decibels.
///
/// Returns `None` when the frequency is not positive and finite.
pub fn gaseous_attenuation(frequency: f64) -> Option<GaseousAttenuation> {
    Some(GaseousAttenuation::new(
        oxygen_zenith_attenuation(frequency)?,
        water_vapour_zenith_attenuation(frequency)?,
    ))
}

/// Returns a Gaussian bump of the given height, centre, and width.
///
/// This is used to shape the absorption features. The width is the half
/// width at which the value falls to `1 / e` of the height.
fn gaussian(frequency: f64, centre: f64, width: f64, height: f64) -> f64 {
    let offset = (frequency - centre) / width;
    height * (-offset * offset).exp()
}
