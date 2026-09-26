//! Free space propagation.

use crate::decibel::Decibel;
use crate::gain::Gain;
use apsis_core::constant::SPEED_OF_LIGHT;
use core::f64::consts::PI;

/// Returns the wavelength of a frequency in vacuum, in meters.
///
/// The relation is `lambda = c / f`, with `c` exact by the definition of the
/// metre.
///
/// Returns `None` when the frequency is not positive and finite.
pub fn wavelength(frequency: f64) -> Option<f64> {
    if frequency > 0.0 && frequency.is_finite() {
        Some(SPEED_OF_LIGHT / frequency)
    } else {
        None
    }
}

/// Returns the free space path loss between two isotropic antennas, as a
/// ratio in decibels.
///
/// The ratio is positive, so it represents a loss: the received power is the
/// transmitted power divided by this ratio. The relation is
///
/// ```text
/// L = (4 pi d / lambda)^2
/// ```
///
/// which is the spreading loss of a spherical wave over a distance `d`
/// combined with the effective aperture of an isotropic receive antenna.
///
/// The loss grows with frequency at a fixed distance, because the effective
/// aperture of the receive antenna falls as the square of the wavelength.
///
/// The distance must be positive. A zero distance is not a case this
/// expression covers: the ratio would be zero, which is infinite attenuation
/// in decibels, and the underlying far field assumption does not hold.
///
/// Returns `None` when the distance is not positive, or when either argument
/// is not finite.
pub fn free_space_path_loss(distance: f64, frequency: f64) -> Option<Decibel> {
    if !distance.is_finite() || distance <= 0.0 {
        return None;
    }
    let lambda = wavelength(frequency)?;
    let ratio = 4.0 * PI * distance / lambda;
    Decibel::from_ratio(ratio * ratio)
}

/// Returns the received power ratio between two antennas in free space.
///
/// The received power is the transmitted power multiplied by the two antenna
/// gains and divided by the free space path loss, which in decibels is the
/// sum
///
/// ```text
/// P_r / P_t = G_t G_r / L
/// ```
///
/// The result is the ratio in decibels, so a positive value means the
/// receive antenna collects more than was transmitted, which happens when
/// the two gains exceed the spreading loss.
///
/// Returns `None` when the distance or frequency is not usable.
pub fn free_space_received_ratio(
    distance: f64,
    frequency: f64,
    transmitter: Gain,
    receiver: Gain,
) -> Option<Decibel> {
    let loss = free_space_path_loss(distance, frequency)?;
    let transmitter = transmitter.as_decibel()?;
    let receiver = receiver.as_decibel()?;
    Some(transmitter + receiver - loss)
}
