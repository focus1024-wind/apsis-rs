//! The link geometry type and the state of a link.

use apsis_math::Vec3;

/// The geometry of a link between two satellites.
///
/// The range is the straight line distance, which a link budget uses for the
/// free space path loss. The two pointing vectors are the directions each
/// terminal must aim, expressed in the frame the positions were given in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkGeometry {
    /// The distance between the two satellites, in kilometers.
    pub range_km: f64,
    /// The unit vector from the first satellite toward the second.
    pub direction_first_to_second: Vec3,
    /// The unit vector from the second satellite toward the first.
    pub direction_second_to_first: Vec3,
    /// The closest the line between the two passes to the centre of the
    /// body, in kilometers.
    ///
    /// This is the quantity that decides whether the link is blocked. It is
    /// reported because a link that clears the limb by a small margin is
    /// fragile: atmospheric refraction and the oblateness of the body both
    /// eat into the margin.
    pub centre_distance_km: f64,
}

impl LinkGeometry {
    /// Returns the grazing margin of the link, in kilometers.
    ///
    /// This is how far the line between the two satellites clears the
    /// surface, measured at the closest point. A negative value means the
    /// link is blocked.
    pub fn surface_clearance(&self, body_radius: f64) -> f64 {
        self.centre_distance_km - body_radius
    }

    /// Returns `true` when the link clears the body.
    pub fn is_clear(&self, body_radius: f64) -> bool {
        self.centre_distance_km >= body_radius
    }
}

/// The state of a link at one instant.
///
/// A link is either carrying traffic or not. The reasons a link can be
/// unavailable are kept separate, because routing across a constellation has
/// to distinguish a link that is geometrically blocked from one that is
/// merely not established yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkState {
    /// The link is geometrically clear and may be used.
    Available,
    /// The Earth blocks the line between the two satellites.
    Blocked,
    /// The Sun is within the field of view of one of the terminals, so an
    /// optical link cannot be used.
    SunInField,
    /// The link is clear but is not established, for example because it has
    /// not yet been acquired or its capacity is exhausted.
    NotEstablished,
}

impl LinkState {
    /// Returns `true` when the link may carry traffic.
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Available)
    }

    /// Returns a short name for the state.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::SunInField => "sun in field",
            Self::NotEstablished => "not established",
        }
    }
}
