//! The Walker constellation pattern and its instantiation.

use crate::error::{ConstellationError, ConstellationErrorKind};
use apsis_kepler::KeplerianElements;
use core::f64::consts::PI;

/// A Walker delta pattern.
///
/// A Walker pattern places a number of satellites evenly around the Earth in
/// a number of planes that share an inclination and a semi-major axis. The
/// relative phasing of adjacent planes is set by the phase factor, so the
/// whole constellation is described by five numbers.
///
/// The pattern is written `i: t/p/f`, where `i` is the inclination, `t` the
/// total number of satellites, `p` the number of planes, and `f` the phase
/// factor. This structure stores the same quantities, with the inclination
/// in radians and the semi-major axis in kilometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkerPattern {
    /// The total number of satellites.
    pub total_satellites: usize,
    /// The number of orbital planes.
    pub planes: usize,
    /// The phase factor, in the range `0..planes`.
    pub phase_factor: usize,
    /// The inclination of every plane, in radians.
    pub inclination: f64,
    /// The semi-major axis of every satellite, in kilometers.
    pub semi_major_axis: f64,
}

impl WalkerPattern {
    /// Returns the number of satellites in each plane.
    ///
    /// Returns `None` when the satellite count is not a multiple of the
    /// plane count.
    pub fn satellites_per_plane(self) -> Option<usize> {
        if self.planes == 0 || self.total_satellites % self.planes != 0 {
            return None;
        }
        Some(self.total_satellites / self.planes)
    }

    /// Returns the spacing between adjacent planes in right ascension, in
    /// radians.
    ///
    /// Returns `None` when the plane count is zero, which is the only case
    /// in which the spacing is undefined.
    ///
    /// The spacing does not depend on the pattern being consistent, so a
    /// caller can report the value it would use even for a pattern that
    /// cannot be instantiated.
    pub fn plane_spacing(self) -> Option<f64> {
        (self.planes > 0).then(|| 2.0 * PI / self.planes as f64)
    }

    /// Returns the phase offset applied between adjacent planes, in radians.
    ///
    /// This is `f * 360 / t` degrees, where `f` is the phase factor and `t`
    /// the total satellite count.
    ///
    /// Returns `None` when the satellite count is zero, which is the only
    /// case in which the offset is undefined.
    pub fn phase_offset(self) -> Option<f64> {
        if self.total_satellites == 0 {
            return None;
        }
        Some(2.0 * PI * self.phase_factor as f64 / self.total_satellites as f64)
    }

    /// Returns `true` when the pattern can be instantiated.
    pub fn is_valid(self) -> bool {
        self.total_satellites > 0
            && self.planes > 0
            && self.total_satellites % self.planes == 0
            && self.phase_factor < self.planes
            && self.inclination.is_finite()
            && (0.0..=PI).contains(&self.inclination)
            && self.semi_major_axis.is_finite()
            && self.semi_major_axis > 0.0
    }

    /// Returns the Walker notation for this pattern, such as `53: 24/3/1`.
    ///
    /// The inclination is rounded to one decimal place, which is the
    /// precision the notation is normally quoted to.
    pub fn notation(self) -> String {
        format!(
            "{}: {}/{}/{}",
            self.inclination.to_degrees(),
            self.total_satellites,
            self.planes,
            self.phase_factor
        )
    }

    /// Instantiates the pattern as a list of orbital elements.
    ///
    /// The mean anomaly of the reference satellite of the first plane is
    /// zero. Each further satellite in a plane is advanced by
    /// `360 / (t / p)` degrees, and each further plane is advanced by the
    /// phase offset.
    ///
    /// Returns an error describing why the pattern cannot be instantiated.
    ///
    /// Source: the phasing rule is the one defined in Walker, J. G.,
    /// *Circular Orbit Patterns Providing Continuous Whole Earth Coverage*,
    /// Royal Aircraft Establishment Technical Report 70211, 1970.
    pub fn instantiate(self) -> Result<Vec<KeplerianElements>, ConstellationError> {
        if self.total_satellites == 0 {
            return Err(ConstellationError::new(
                ConstellationErrorKind::NoSatellites,
            ));
        }
        if self.planes == 0 {
            return Err(ConstellationError::new(ConstellationErrorKind::NoPlanes));
        }
        let per_plane = self.satellites_per_plane().ok_or_else(|| {
            ConstellationError::new(ConstellationErrorKind::UnevenPlanes).with_detail(format!(
                "{} satellites cannot be spread over {} planes",
                self.total_satellites, self.planes
            ))
        })?;
        if !self.is_valid() {
            return Err(
                ConstellationError::new(ConstellationErrorKind::InvalidOrbit).with_detail(format!(
                    "inclination {} rad, semi-major axis {} km, phase factor {}",
                    self.inclination, self.semi_major_axis, self.phase_factor
                )),
            );
        }

        let plane_spacing = self.plane_spacing().expect("a positive plane count");
        let phase_offset = self.phase_offset().expect("a positive satellite count");
        let anomaly_spacing = 2.0 * PI / per_plane as f64;

        let mut satellites = Vec::with_capacity(self.total_satellites);
        for plane in 0..self.planes {
            // The right ascension of this plane.
            let right_ascension = (plane as f64 * plane_spacing).rem_euclid(2.0 * PI);
            // The phase offset of this plane relative to the first.
            let plane_phase = plane as f64 * phase_offset;
            for satellite in 0..per_plane {
                let mean_anomaly =
                    (satellite as f64 * anomaly_spacing + plane_phase).rem_euclid(2.0 * PI);
                satellites.push(KeplerianElements {
                    semi_major_axis: self.semi_major_axis,
                    // The Walker pattern is circular.
                    eccentricity: 0.0,
                    inclination: self.inclination,
                    right_ascension,
                    // The argument of perigee is undefined for a circular
                    // orbit, so it is set to zero and the position along the
                    // orbit is carried entirely by the mean anomaly.
                    argument_of_perigee: 0.0,
                    true_anomaly: apsis_kepler::true_from_mean(mean_anomaly, 0.0),
                });
            }
        }
        Ok(satellites)
    }
}

/// One satellite in an instantiated constellation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Satellite {
    /// The index of the satellite within the constellation.
    pub index: usize,
    /// The index of the orbital plane it belongs to.
    pub plane: usize,
    /// The index of the satellite within its plane.
    pub index_in_plane: usize,
    /// The orbital elements at the reference time.
    pub elements: KeplerianElements,
}

/// A constellation instantiated at a reference time.
///
/// The elements are the ones at the reference time passed to the
/// constructor. Advancing them in time is the job of the orbit layer, which
/// propagates each satellite independently.
#[derive(Debug, Clone, PartialEq)]
pub struct Constellation {
    satellites: Vec<Satellite>,
    pattern: Option<WalkerPattern>,
    reference_mjd: f64,
}

impl Constellation {
    /// Instantiates a Walker pattern at a reference time.
    ///
    /// The reference time is a Modified Julian Date on the TAI scale. It is
    /// stored with the constellation and returned unchanged; the elements
    /// are the ones at that instant.
    ///
    /// Returns an error when the pattern cannot be instantiated.
    pub fn walker(pattern: WalkerPattern, reference_mjd: f64) -> Result<Self, ConstellationError> {
        if !reference_mjd.is_finite() {
            return Err(
                ConstellationError::new(ConstellationErrorKind::InvalidOrbit)
                    .with_detail("the reference time is not finite"),
            );
        }
        let elements = pattern.instantiate()?;
        let per_plane = pattern
            .satellites_per_plane()
            .expect("instantiation checked the division");

        let satellites = elements
            .into_iter()
            .enumerate()
            .map(|(index, elements)| Satellite {
                // The satellites are produced plane by plane, so the plane
                // index is the quotient and the position within it the
                // remainder.
                index,
                plane: index / per_plane,
                index_in_plane: index % per_plane,
                elements,
            })
            .collect();

        Ok(Self {
            satellites,
            pattern: Some(pattern),
            reference_mjd,
        })
    }

    /// Builds a constellation from a list of satellites.
    ///
    /// This is the constructor for a constellation that does not follow the
    /// Walker pattern. The plane index of each satellite is derived from the
    /// right ascension of its elements, so satellites sharing a plane are
    /// grouped.
    ///
    /// Returns an error when the list is empty, when the reference time is
    /// not finite, or when any element set is not a valid orbit.
    pub fn from_satellites(
        elements: &[KeplerianElements],
        reference_mjd: f64,
    ) -> Result<Self, ConstellationError> {
        if elements.is_empty() {
            return Err(ConstellationError::new(
                ConstellationErrorKind::NoSatellites,
            ));
        }
        if !reference_mjd.is_finite() {
            return Err(
                ConstellationError::new(ConstellationErrorKind::InvalidOrbit)
                    .with_detail("the reference time is not finite"),
            );
        }

        // Group by right ascension, assigning plane indices in order of
        // first appearance so that the numbering is stable.
        let mut planes: Vec<f64> = Vec::new();
        let mut satellites = Vec::with_capacity(elements.len());
        let mut counts: Vec<usize> = Vec::new();
        for (index, elements) in elements.iter().enumerate() {
            if !elements.is_valid() {
                return Err(
                    ConstellationError::new(ConstellationErrorKind::InvalidOrbit)
                        .with_detail(format!("satellite {index} has invalid elements")),
                );
            }
            let plane = match planes
                .iter()
                .position(|value| (value - elements.right_ascension).abs() < 1e-12)
            {
                Some(plane) => plane,
                None => {
                    planes.push(elements.right_ascension);
                    counts.push(0);
                    planes.len() - 1
                }
            };
            let index_in_plane = counts[plane];
            counts[plane] += 1;
            satellites.push(Satellite {
                index,
                plane,
                index_in_plane,
                elements: *elements,
            });
        }

        Ok(Self {
            satellites,
            pattern: None,
            reference_mjd,
        })
    }

    /// Returns the satellites in the constellation.
    pub fn satellites(&self) -> &[Satellite] {
        &self.satellites
    }

    /// Returns the number of satellites.
    pub fn len(&self) -> usize {
        self.satellites.len()
    }

    /// Returns `true` when the constellation is empty.
    ///
    /// A constellation built through either constructor is never empty, so
    /// this always returns `false`; it exists because a collection type
    /// should offer it alongside [`Constellation::len`].
    pub fn is_empty(&self) -> bool {
        self.satellites.is_empty()
    }

    /// Returns the number of orbital planes.
    pub fn plane_count(&self) -> usize {
        self.satellites
            .iter()
            .map(|satellite| satellite.plane)
            .max()
            .map_or(0, |highest| highest + 1)
    }

    /// Returns the Walker pattern, when the constellation was built from
    /// one.
    pub fn pattern(&self) -> Option<WalkerPattern> {
        self.pattern
    }

    /// Returns the reference time as a Modified Julian Date on the TAI
    /// scale.
    pub fn reference_mjd(&self) -> f64 {
        self.reference_mjd
    }

    /// Returns the satellites of one plane.
    ///
    /// Returns an empty slice when the plane index is out of range.
    pub fn plane(&self, plane: usize) -> Vec<Satellite> {
        self.satellites
            .iter()
            .filter(|satellite| satellite.plane == plane)
            .copied()
            .collect()
    }

    /// Returns the altitude of the constellation above the given radius, in
    /// kilometers.
    ///
    /// Every satellite of a Walker pattern shares a semi-major axis, so a
    /// single altitude describes the whole constellation. For a custom
    /// constellation this is the altitude of the first satellite.
    ///
    /// Returns `None` when the constellation is empty.
    pub fn altitude_above(&self, body_radius_km: f64) -> Option<f64> {
        self.satellites
            .first()
            .map(|satellite| satellite.elements.semi_major_axis - body_radius_km)
    }

    /// Returns the count of satellites in each plane.
    ///
    /// The result has one entry per plane, in plane order.
    pub fn plane_populations(&self) -> Vec<usize> {
        let mut populations = vec![0; self.plane_count()];
        for satellite in &self.satellites {
            populations[satellite.plane] += 1;
        }
        populations
    }
}
