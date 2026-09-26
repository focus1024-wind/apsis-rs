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
    let (cap_angle, floor_angle) = if large_aperture {
        (1.0_f64.to_radians(), 36.0_f64.to_radians())
    } else {
        (1.0_f64.to_radians(), 48.0_f64.to_radians())
    };
    let head = if large_aperture { 29.0 } else { 32.0 };

    if off_axis_angle < cap_angle {
        // Inside the first degree the pattern is taken at its value at one
        // degree, which is the most restrictive point on the falling branch.
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pattern {
    /// A parabolic aperture, described by its diameter and efficiency.
    Parabolic {
        /// The reflector diameter, in meters.
        diameter: f64,
        /// The aperture efficiency, dimensionless.
        efficiency: f64,
    },
    /// An antenna described directly by its peak gain and half power beam
    /// width.
    Direct {
        /// The peak gain on boresight.
        peak: Gain,
        /// The half power beam width, in radians.
        beam_width: f64,
    },
}

impl Pattern {
    /// Returns the peak gain of this pattern at the given wavelength.
    ///
    /// The wavelength is ignored for a directly specified pattern, which
    /// already carries its peak gain.
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElementPattern {
    /// The peak gain on boresight.
    pub peak: Gain,
    /// The half power beam width, in radians.
    pub beam_width: f64,
    /// The sidelobe floor, in decibels below the peak.
    pub floor_db: f64,
}

impl Default for ElementPattern {
    /// Returns a pattern for an isotropic antenna, with a floor that makes
    /// the attenuation zero in every direction.
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
    pub fn from_parabolic(aperture: ParabolicAperture, wavelength: f64) -> Option<Self> {
        Some(Self {
            peak: aperture.peak_gain(wavelength)?,
            beam_width: aperture.half_power_beam_width(wavelength)?,
            floor_db: 32.0,
        })
    }

    /// Returns this pattern with a different sidelobe floor.
    pub const fn with_floor(mut self, floor_db: f64) -> Self {
        self.floor_db = floor_db;
        self
    }

    /// Returns the gain at an angle from boresight.
    ///
    /// Returns `None` when the pattern is not usable or the angle is not
    /// finite.
    pub fn gain_at(self, off_axis_angle: f64) -> Option<Gain> {
        let attenuation =
            off_axis_attenuation(self.peak, self.beam_width, off_axis_angle, self.floor_db)?;
        Gain::from_dbi(attenuation.db())
    }

    /// Returns `true` when the beam width and floor describe a usable
    /// pattern.
    pub fn is_valid(self) -> bool {
        self.peak.dbi().is_some()
            && self.beam_width.is_finite()
            && self.beam_width > 0.0
            && self.floor_db.is_finite()
            && self.floor_db > 0.0
    }
}
