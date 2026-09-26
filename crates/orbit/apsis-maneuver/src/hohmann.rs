//! The two impulse Hohmann transfer between coplanar circular orbits.

use crate::error::{ManeuverError, ManeuverErrorKind};
use apsis_kepler::GravityParameter;

/// A two impulse transfer between two coplanar circular orbits.
///
/// The transfer is the classic Hohmann maneuver: a burn onto an ellipse
/// whose perigee touches the lower orbit and whose apogee touches the upper
/// one, a coast for half the ellipse's period, and a second burn onto the
/// destination circle. It is the cheapest two impulse transfer between two
/// coplanar circular orbits in the impulsive model, which is why it is the
/// reference every other transfer is measured against.
///
/// The burns are signed along the direction of motion: a positive value
/// raises the orbit and a negative one lowers it, so the same type covers a
/// climb and a descent. The total is the sum of the magnitudes, which is the
/// quantity a propellant budget uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HohmannTransfer {
    /// The radius of the orbit the transfer starts from, in kilometers.
    pub from_radius: f64,
    /// The radius of the orbit the transfer arrives at, in kilometers.
    pub to_radius: f64,
    /// The semi-major axis of the transfer ellipse, in kilometers.
    pub transfer_semi_major_axis: f64,
    /// The first burn, along the direction of motion, in kilometers per
    /// second.
    pub first_burn: f64,
    /// The second burn, along the direction of motion, in kilometers per
    /// second.
    pub second_burn: f64,
    /// The coast time between the burns, in seconds.
    pub transfer_time: f64,
}

impl HohmannTransfer {
    /// Returns the total velocity change, in kilometers per second.
    ///
    /// This is the sum of the magnitudes of the two burns, which is what the
    /// transfer costs whatever its direction.
    pub fn total_delta_v(&self) -> f64 {
        self.first_burn.abs() + self.second_burn.abs()
    }

    /// Returns `true` when the transfer raises the orbit.
    pub fn is_raising(&self) -> bool {
        self.to_radius > self.from_radius
    }

    /// Returns the velocity change of the first burn, unsigned.
    pub fn first_burn_magnitude(&self) -> f64 {
        self.first_burn.abs()
    }

    /// Returns the velocity change of the second burn, unsigned.
    pub fn second_burn_magnitude(&self) -> f64 {
        self.second_burn.abs()
    }

    /// Returns the mean radius the transfer is symmetric about, in
    /// kilometers.
    ///
    /// This is the semi-major axis of the ellipse, which is the arithmetic
    /// mean of the two orbit radii.
    pub fn mean_radius(&self) -> f64 {
        self.transfer_semi_major_axis
    }
}

/// Returns the two impulse transfer between two circular orbits.
///
/// The radii are in kilometers and must be finite and positive. The two may
/// be given in either order: a larger second radius is a climb and a smaller
/// one a descent.
///
/// Returns an error when a radius is not usable or when the two are equal,
/// because a transfer between an orbit and itself is not a maneuver and is
/// almost always a mistake in a plan.
///
/// The burns follow from the vis-viva equation. At the first radius the
/// circular speed is `sqrt(mu / r1)` and the speed on the transfer ellipse
/// is `sqrt(mu (2 / r1 - 1 / a))`, where `a` is the semi-major axis of the
/// ellipse, so the first burn is the difference. The second burn is the
/// difference between the circular speed at the destination and the speed on
/// the ellipse there, taken the other way round because the ellipse arrives
/// slower than the circle it must join. The coast time is half the period of
/// the ellipse.
pub fn hohmann_transfer(
    from_radius: f64,
    to_radius: f64,
    parameter: GravityParameter,
) -> Result<HohmannTransfer, ManeuverError> {
    for radius in [from_radius, to_radius] {
        if !radius.is_finite() || radius <= 0.0 {
            return Err(ManeuverError::new(ManeuverErrorKind::InvalidRadius)
                .with_detail(format!("radius {radius}")));
        }
    }
    if from_radius == to_radius {
        return Err(ManeuverError::new(ManeuverErrorKind::DegenerateTransfer)
            .with_detail(format!("both radii are {from_radius}")));
    }

    let mu = parameter.value();
    let transfer_semi_major_axis = (from_radius + to_radius) / 2.0;

    // The circular speeds at each radius.
    let circular_from = (mu / from_radius).sqrt();
    let circular_to = (mu / to_radius).sqrt();

    // The speeds on the transfer ellipse at each radius, from the vis-viva
    // equation `v^2 = mu (2 / r - 1 / a)`.
    let transfer_from = (mu * (2.0 / from_radius - 1.0 / transfer_semi_major_axis)).sqrt();
    let transfer_to = (mu * (2.0 / to_radius - 1.0 / transfer_semi_major_axis)).sqrt();

    // The first burn accelerates onto the ellipse; the second accelerates
    // from the ellipse speed up to the circular speed at the destination.
    // Both are along the direction of motion, and the sign follows from
    // which speed is larger.
    let first_burn = transfer_from - circular_from;
    let second_burn = circular_to - transfer_to;

    // Half the period of the transfer ellipse.
    let transfer_time = core::f64::consts::PI * (transfer_semi_major_axis.powi(3) / mu).sqrt();

    Ok(HohmannTransfer {
        from_radius,
        to_radius,
        transfer_semi_major_axis,
        first_burn,
        second_burn,
        transfer_time,
    })
}
