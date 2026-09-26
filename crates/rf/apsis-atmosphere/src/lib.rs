//! Atmospheric attenuation and refraction models.
//!
//! A satellite signal passing through the atmosphere is weakened by two
//! mechanisms that are treated separately because they behave differently
//! with frequency, elevation, and weather.
//!
//! Gaseous absorption comes from the absorption lines of oxygen and water
//! vapour. Oxygen has a dense group of lines between 50 and 70 GHz and an
//! isolated line at 118.75 GHz, and water vapour has a strong line at
//! 22.235 GHz and a much stronger one at 183.31 GHz. The attenuation is the
//! line-by-line sum over those lines, which makes it negligible below about
//! 10 GHz and severe in the absorption bands.
//!
//! Rain attenuation is caused by scattering and absorption in
//! precipitation. It is negligible below about 4 GHz, grows as a power of
//! the rain rate, and depends strongly on elevation and polarisation.
//!
//! Both are functions of the elevation angle through the same path
//! geometry: a signal at a low elevation crosses more atmosphere.
//!
//! Sources:
//!
//! - Gaseous specific attenuation and equivalent heights: ITU-R
//!   Recommendation P.676-12, *Attenuation by atmospheric gases*.
//! - Rain specific attenuation coefficients: ITU-R Recommendation P.838-3,
//!   *Specific attenuation model for rain for use in prediction methods*.
//! - Rain path and elevation dependence: ITU-R Recommendation P.618-13,
//!   *Propagation data and prediction methods required for the design of
//!   Earth-space telecommunication systems*.
//!
//! ```
//! use apsis_atmosphere::{Atmosphere, slant_path_attenuation, zenith_attenuation};
//!
//! // At the reference atmosphere the 60 GHz oxygen complex is heavily
//! // attenuated while a Ku band link is barely affected.
//! let ku = zenith_attenuation(12.0, Atmosphere::REFERENCE).expect("valid");
//! let oxygen = zenith_attenuation(60.0, Atmosphere::REFERENCE).expect("valid");
//! assert!(ku < 0.1);
//! assert!(oxygen > 100.0);
//!
//! // A lower elevation crosses more atmosphere.
//! let zenith = zenith_attenuation(12.0, Atmosphere::REFERENCE).expect("valid");
//! let low = slant_path_attenuation(12.0, 0.2, Atmosphere::REFERENCE).expect("valid");
//! assert!(low > zenith);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod gaseous;
mod gaseous_path;
mod geometry;
mod p839;
mod p839_table;
mod rain;

pub use gaseous::{
    Atmosphere, specific_attenuation, specific_attenuation_dry_air,
    specific_attenuation_water_vapour,
};
pub use gaseous_path::{
    EquivalentHeights, equivalent_heights, slant_path_attenuation, zenith_attenuation,
};
pub use geometry::{air_mass, elevation_from_zenith, zenith_to_elevation};
pub use p839::{RAIN_HEIGHT_OFFSET_KM, grid_step_degrees, rain_height, zero_isotherm_height};
pub use rain::{
    DEFAULT_RAIN_HEIGHT_KM, RainAttenuation, polarisation_tilt, rain_attenuation,
    rain_coefficients, specific_rain_attenuation,
};

pub use apsis_rf::Decibel;
