//! Atmospheric attenuation and refraction models.
//!
//! A satellite signal passing through the atmosphere is weakened by two
//! mechanisms that are treated separately because they behave differently
//! with frequency, elevation, and weather.
//!
//! Gaseous absorption comes from oxygen and water vapour. Oxygen has
//! absorption lines near 60 GHz and 118 GHz and water vapour near 22 GHz, so
//! the effect is small below about 10 GHz and grows rapidly above 20 GHz.
//! It depends on the elevation angle through the air mass along the path.
//!
//! Rain attenuation is caused by scattering and absorption in precipitation.
//! It is negligible below about 4 GHz, grows roughly as a power of the rain
//! rate with an exponent near one, and depends strongly on elevation.
//!
//! Both are functions of the elevation angle through the same path
//! geometry: a signal at a low elevation crosses more atmosphere. The air
//! mass for a flat atmosphere is `1 / sin(theta)` for elevation `theta`,
//! and this crate uses that form with an optional correction for the
//! curvature of the Earth.
//!
//! Sources:
//!
//! - Gaseous attenuation: ITU-R Recommendation P.676, *Attenuation by
//!   atmospheric gases*.
//! - Rain specific attenuation coefficients: ITU-R Recommendation P.838,
//!   *Specific attenuation model for rain for use in prediction methods*.
//! - Rain path and elevation dependence: ITU-R Recommendation P.618,
//!   *Propagation data and prediction methods required for the design of
//!   Earth-space telecommunication systems*.
//!
//! ```
//! use apsis_atmosphere::{air_mass, elevation_from_zenith, zenith_to_elevation};
//!
//! // At the zenith the path is the shortest.
//! assert_eq!(air_mass(zenith_to_elevation(0.0)), 1.0);
//!
//! // At thirty degrees elevation the air mass is twice as large, because
//! // the cosecant of thirty degrees is two.
//! let thirty = core::f64::consts::PI / 6.0;
//! assert!((air_mass(thirty) - 2.0).abs() < 1e-9);
//!
//! // Elevation and zenith angle are complementary.
//! assert!((elevation_from_zenith(thirty) + thirty - core::f64::consts::FRAC_PI_2).abs() < 1e-12);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod gaseous;
mod geometry;
mod rain;

pub use gaseous::{
    GaseousAttenuation, gaseous_attenuation, oxygen_zenith_attenuation,
    water_vapour_zenith_attenuation,
};
pub use geometry::{air_mass, elevation_from_zenith, zenith_to_elevation};
pub use rain::{
    RainAttenuation, polarisation_tilt, rain_attenuation, rain_coefficients,
    specific_rain_attenuation,
};

pub use apsis_rf::Decibel;
