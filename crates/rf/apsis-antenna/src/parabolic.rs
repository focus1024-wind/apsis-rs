//! The parabolic aperture model.

use apsis_rf::Gain;
use core::f64::consts::PI;

/// A parabolic reflector described by its diameter and aperture efficiency.
///
/// The aperture efficiency collects every loss between the power delivered
/// to the feed and the power that leaves the aperture as a uniform plane
/// wave: illumination taper, spillover, blockage by the feed and its
/// supports, and surface error. A well adjusted dish reaches 0.65 to 0.70,
/// and a small or simply built one is nearer 0.55.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParabolicAperture {
    /// The reflector diameter, in meters.
    pub diameter: f64,
    /// The aperture efficiency, dimensionless, in `(0, 1]`.
    pub efficiency: f64,
}

impl ParabolicAperture {
    /// Returns the physical area of the aperture, in square meters.
    pub fn area(self) -> f64 {
        PI * (self.diameter / 2.0).powi(2)
    }

    /// Returns the effective area, in square meters.
    ///
    /// This is the physical area scaled by the aperture efficiency. It is
    /// the area that, if it collected uniformly, would deliver the observed
    /// power.
    pub fn effective_area(self) -> f64 {
        self.area() * self.efficiency
    }

    /// Returns `true` when the aperture describes a usable dish.
    pub fn is_valid(self) -> bool {
        self.diameter.is_finite()
            && self.diameter > 0.0
            && self.efficiency.is_finite()
            && self.efficiency > 0.0
            && self.efficiency <= 1.0
    }

    /// Returns the peak gain on boresight at the given wavelength.
    ///
    /// The relation is
    ///
    /// ```text
    /// G = eta (pi D / lambda)^2
    /// ```
    ///
    /// which follows from the effective area through
    /// [`Gain::from_effective_aperture`]. Returns `None` for an invalid
    /// aperture or wavelength.
    pub fn peak_gain(self, wavelength: f64) -> Option<Gain> {
        if !self.is_valid() {
            return None;
        }
        Gain::from_effective_aperture(self.effective_area(), wavelength)
    }

    /// Returns the half power beam width at the given wavelength, in
    /// radians.
    ///
    /// This uses the uniform illumination relation, which is the narrowest
    /// a dish of this diameter can be. A real dish with an illumination
    /// taper is wider; use [`ParabolicAperture::half_power_beam_width`] to
    /// include the taper.
    ///
    /// Returns `None` for an invalid aperture or wavelength.
    pub fn uniform_beam_width(self, wavelength: f64) -> Option<f64> {
        if !self.is_valid() {
            return None;
        }
        beam_width(self.diameter, wavelength, 1.02)
    }

    /// Returns the half power beam width at the given wavelength, in
    /// radians, allowing for an illumination taper.
    ///
    /// A tapered illumination reduces the sidelobes at the cost of a wider
    /// main lobe. The factor `k` in `theta = k lambda / D` rises from about
    /// 1.02 for uniform illumination to about 1.2 for the taper that
    /// minimises sidelobes. A value of 1.15 is representative of a
    /// well designed dish.
    ///
    /// Returns `None` for an invalid aperture or wavelength.
    pub fn half_power_beam_width(self, wavelength: f64) -> Option<f64> {
        if !self.is_valid() {
            return None;
        }
        beam_width(self.diameter, wavelength, 1.15)
    }
}

/// Returns the half power beam width of a circular aperture, in radians.
///
/// The relation is `theta = k lambda / D`, where `k` depends on how the
/// aperture is illuminated. Uniform illumination gives the narrowest beam,
/// with `k` about 1.02, and the taper that suppresses sidelobes gives the
/// widest, with `k` about 1.2.
///
/// The result is the full width between the two three decibel points, not
/// the half width. It is a small angle approximation, so the caller should
/// confirm that the resulting width is small compared to a radian.
///
/// Returns `None` when the diameter or wavelength is not positive, or when
/// `k` is not positive.
pub fn beam_width(diameter: f64, wavelength: f64, k: f64) -> Option<f64> {
    if !diameter.is_finite()
        || diameter <= 0.0
        || !wavelength.is_finite()
        || wavelength <= 0.0
        || !k.is_finite()
        || k <= 0.0
    {
        return None;
    }
    Some(k * wavelength / diameter)
}

/// Returns the angle at which the squared angle law of
/// [`off_axis_attenuation`](crate::off_axis_attenuation) reaches the
/// sidelobe floor.
///
/// Beyond this angle the model holds the gain at the floor rather than
/// letting it fall without bound. The angle follows from setting the
/// attenuation `12 (theta / theta_0)^2` equal to the floor, which gives
/// `theta = theta_0 sqrt(floor / 12)`.
///
/// Returns `None` when the beam width or floor is not positive and finite.
pub fn floor_angle(beam_width: f64, floor_db: f64) -> Option<f64> {
    if !beam_width.is_finite() || beam_width <= 0.0 {
        return None;
    }
    if !floor_db.is_finite() || floor_db <= 0.0 {
        return None;
    }
    Some(beam_width * (floor_db / 12.0).sqrt())
}
