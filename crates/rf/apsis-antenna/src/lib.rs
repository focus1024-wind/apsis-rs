//! Antenna radiation patterns and gain models.
//!
//! An antenna does not radiate equally in every direction. This crate models
//! how the gain varies with angle for the two shapes that appear in almost
//! every satellite link: a parabolic reflector, whose beam is a narrow
//! pencil, and an idealised element with a broad pattern.
//!
//! Three beam width conventions are used in practice and they are not
//! interchangeable:
//!
//! - The half power beam width is the angle between the two directions where
//!   the gain has fallen to half its peak, which is three decibels down.
//! - The null to null beam width is the angle between the first two zeros on
//!   either side of the main lobe. It is roughly twice the half power width
//!   for a uniformly illuminated aperture.
//! - The first sidelobe level is the peak gain outside the main lobe. For a
//!   uniformly illuminated circular aperture it is about minus 17.6 dB.
//!
//! Every function here takes or returns angles in radians and gains as
//! [`Gain`], so a result can be fed straight into a link budget.
//!
//! Sources:
//!
//! - Parabolic aperture gain: `G = eta (pi D / lambda)^2`, where `eta` is
//!   the aperture efficiency, typically between 0.55 and 0.70 for a
//!   well adjusted dish.
//! - Half power beam width of a circular aperture: `theta = k lambda / D`
//!   with `k` about 1.02 for uniform illumination and close to 1.2 for
//!   typical illumination tapers.
//! - The co-polar reference pattern of a simple Earth station antenna, which
//!   falls as the square of the off-axis angle down to a sidelobe floor. This
//!   is the form used by ITU-R Recommendation S.465 to bound interference
//!   from one satellite system into another.
//!
//! ```
//! use apsis_antenna::ParabolicAperture;
//!
//! // A three meter dish at twelve gigahertz has a peak gain near 50 dBi.
//! let dish = ParabolicAperture {
//!     diameter: 3.0,
//!     efficiency: 0.65,
//! };
//! let peak = dish.peak_gain(0.025).expect("valid wavelength");
//! assert!(peak.dbi().expect("positive") > 45.0);
//! assert!(peak.dbi().expect("positive") < 50.0);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod parabolic;
mod pattern;

pub use parabolic::{ParabolicAperture, beam_width, floor_angle};
pub use pattern::{ElementPattern, Pattern, mask_gain, off_axis_attenuation};
