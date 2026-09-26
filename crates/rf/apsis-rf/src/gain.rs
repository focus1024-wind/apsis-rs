//! Antenna gain.

use crate::decibel::Decibel;
use core::f64::consts::PI;
use core::fmt;

/// The gain of a half wave dipole relative to an isotropic radiator, in
/// decibels.
///
/// The exact directivity of an ideal half wave dipole is 1.6409, which is
/// 2.1509 dB. Engineering practice rounds this to 2.15 dB, and that rounded
/// value is what the `dBd` and `dBi` conventions are defined against. This
/// crate uses the conventional value so that a conversion between the two
/// named forms is exact, and it derives the linear ratio from it.
pub const DIPOLE_OVER_ISOTROPIC_DB: f64 = 2.15;

/// The linear gain ratio of a half wave dipole, `10^(2.15 / 10)`.
///
/// This is about 1.64059, within 0.02 percent of the exact directivity of an
/// ideal dipole, which is 1.6409.
pub const HALF_WAVE_DIPOLE_RATIO: f64 = 1.640_589_773_199_539_4;

/// An antenna gain expressed as a ratio to an isotropic radiator.
///
/// The type stores the linear ratio. The two named decibel forms are offered
/// as conversions, because the choice is a matter of convention rather than
/// of the gain itself:
///
/// - `dBi` is the ratio to an isotropic radiator, which radiates equally in
///   every direction.
/// - `dBd` is the ratio to a half wave dipole, which is
///   [`DIPOLE_OVER_ISOTROPIC_DB`] decibels above an isotropic radiator.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Gain(f64);

impl Default for Gain {
    /// Returns unit gain, matching [`Gain::ISOTROPIC`].
    ///
    /// This is written by hand rather than derived, because the default for
    /// a ratio type is not its zero but its unit.
    fn default() -> Self {
        Self::ISOTROPIC
    }
}

impl Gain {
    /// Unit gain, as an isotropic radiator has.
    pub const ISOTROPIC: Self = Self(1.0);

    /// The gain of a half wave dipole, about 1.6406.
    pub const HALF_WAVE_DIPOLE: Self = Self(HALF_WAVE_DIPOLE_RATIO);

    /// Creates a gain from a linear ratio.
    pub const fn from_ratio(ratio: f64) -> Self {
        Self(ratio)
    }

    /// Creates a gain from a value in decibels relative to an isotropic
    /// radiator.
    ///
    /// A gain ratio is never negative, so any finite reading is accepted.
    pub fn from_dbi(dbi: f64) -> Option<Self> {
        if !dbi.is_finite() {
            return None;
        }
        Some(Self(Decibel::from_db(dbi).ratio()))
    }

    /// Creates a gain from a value in decibels relative to a half wave
    /// dipole.
    ///
    /// Returns `None` when the reading is not finite.
    pub fn from_dbd(dbd: f64) -> Option<Self> {
        if !dbd.is_finite() {
            return None;
        }
        Self::from_dbi(dbd + DIPOLE_OVER_ISOTROPIC_DB)
    }

    /// Returns the linear ratio to an isotropic radiator.
    pub const fn ratio(self) -> f64 {
        self.0
    }

    /// Returns this gain in decibels relative to an isotropic radiator.
    ///
    /// Returns `None` when the ratio is not positive.
    pub fn dbi(self) -> Option<f64> {
        if self.0 > 0.0 && self.0.is_finite() {
            Some(10.0 * self.0.log10())
        } else {
            None
        }
    }

    /// Returns this gain in decibels relative to a half wave dipole.
    ///
    /// Returns `None` when the ratio is not positive.
    pub fn dbd(self) -> Option<f64> {
        self.dbi().map(|dbi| dbi - DIPOLE_OVER_ISOTROPIC_DB)
    }

    /// Returns this gain as a ratio in decibels.
    ///
    /// Returns `None` when the ratio is not positive.
    pub fn as_decibel(self) -> Option<Decibel> {
        Decibel::from_ratio(self.0)
    }

    /// Returns the effective aperture of an antenna with this gain at the
    /// given wavelength, in square meters.
    ///
    /// The relation is `A = G lambda^2 / (4 pi)`. It follows from the
    /// requirement that an antenna in a thermal bath receive as much power
    /// as it radiates.
    ///
    /// Returns `None` when the wavelength is not positive and finite.
    pub fn effective_aperture(self, wavelength: f64) -> Option<f64> {
        if wavelength > 0.0 && wavelength.is_finite() && self.0.is_finite() {
            Some(self.0 * wavelength * wavelength / (4.0 * PI))
        } else {
            None
        }
    }

    /// Returns the gain of an antenna with this effective aperture at the
    /// given wavelength.
    ///
    /// This is the inverse of [`Gain::effective_aperture`].
    ///
    /// Returns `None` when the wavelength is not positive and finite, or
    /// when the aperture is negative.
    pub fn from_effective_aperture(aperture: f64, wavelength: f64) -> Option<Self> {
        if wavelength > 0.0 && wavelength.is_finite() && aperture >= 0.0 && aperture.is_finite() {
            Some(Self(4.0 * PI * aperture / (wavelength * wavelength)))
        } else {
            None
        }
    }
}

impl fmt::Display for Gain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dbi() {
            Some(dbi) => write!(f, "{dbi} dBi"),
            None => write!(f, "{} (ratio)", self.0),
        }
    }
}
