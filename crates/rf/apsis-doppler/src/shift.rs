//! The Doppler shift and the range rate that causes it.

use apsis_core::constant::SPEED_OF_LIGHT;
use apsis_math::Vec3;
use core::fmt;

/// Returns the range rate between two terminals, in meters per second.
///
/// The range rate is the component of the relative velocity along the line
/// between the terminals. A positive value means the separation is
/// increasing.
///
/// The first argument is the position of the remote terminal relative to the
/// local one, so that its magnitude is the separation. The second is the
/// velocity of the remote terminal relative to the local one. Both are in
/// SI units.
///
/// Returns `None` when the separation is zero, because no line of sight is
/// defined there, or when either argument is not finite.
pub fn range_rate_from_velocity(relative_position: Vec3, relative_velocity: Vec3) -> Option<f64> {
    if !relative_position.is_finite() || !relative_velocity.is_finite() {
        return None;
    }
    let range = relative_position.norm();
    if range == 0.0 {
        return None;
    }
    Some(relative_position.dot(relative_velocity) / range)
}

/// Returns the classical Doppler shift of a frequency for a range rate, in
/// hertz.
///
/// The relation is
///
/// ```text
/// delta f = -f_0 (v_r / c)
/// ```
///
/// where `v_r` is the range rate, positive when receding. The shift is
/// negative for a receding terminal, so the received frequency is lower than
/// the transmitted one.
///
/// The magnitude of the shift is small compared to the carrier: a range rate
/// of 7 km/s at 12 GHz gives about 280 kHz, which is two parts in a hundred
/// thousand. The caller must therefore treat the frequency as a double, and
/// the shifted frequency is returned separately rather than by adding a
/// small shift to a large number.
///
/// Returns `None` when the frequency is not positive and finite, or when the
/// range rate is not finite.
pub fn classical_shift(carrier_frequency: f64, range_rate: f64) -> Option<f64> {
    if !carrier_frequency.is_finite() || carrier_frequency <= 0.0 {
        return None;
    }
    if !range_rate.is_finite() {
        return None;
    }
    Some(-carrier_frequency * range_rate / SPEED_OF_LIGHT)
}

/// Returns the relativistic Doppler factor for a range rate.
///
/// The factor is `sqrt((1 - beta) / (1 + beta))` with `beta = v_r / c`.
/// Multiplying the transmitted frequency by it gives the received frequency
/// to all orders in `beta`, so it is not a small correction to the classical
/// shift but the quantity the classical shift approximates.
///
/// Its expansion is `1 - beta + beta^2 / 2 - ...`. The first two terms are
/// the classical result, which is `1 - beta` to within a factor that is
/// itself second order, and the remaining terms are the difference between
/// the two treatments. For a satellite at orbital velocity, `beta` is about
/// `2.3e-5` and the second order term is about `2.7e-10` of the carrier, so
/// the two agree to a few parts in ten to the tenth.
///
/// Returns `None` when the range rate is not finite, or when its magnitude
/// reaches the speed of light, where the expression has no real value.
pub fn relativistic_doppler_factor(range_rate: f64) -> Option<f64> {
    if !range_rate.is_finite() {
        return None;
    }
    let beta = range_rate / SPEED_OF_LIGHT;
    if beta.abs() >= 1.0 {
        return None;
    }
    Some(((1.0 - beta) / (1.0 + beta)).sqrt())
}

/// A Doppler shift applied to a carrier.
///
/// The type carries the carrier frequency and the range rate, so the
/// transmitted and received frequencies and the shift itself are all
/// available without ambiguity about which sign convention was used.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DopplerShift {
    /// The transmitted frequency, in hertz.
    carrier: f64,
    /// The range rate, in meters per second, positive when receding.
    range_rate: f64,
}

impl DopplerShift {
    /// Creates a shift from a carrier frequency and a range rate.
    ///
    /// Returns `None` when the frequency is not positive and finite, or when
    /// the range rate is not finite.
    pub fn from_range_rate(carrier_frequency: f64, range_rate: f64) -> Option<Self> {
        classical_shift(carrier_frequency, range_rate)?;
        Some(Self {
            carrier: carrier_frequency,
            range_rate,
        })
    }

    /// Returns the transmitted frequency, in hertz.
    pub const fn carrier_frequency(self) -> f64 {
        self.carrier
    }

    /// Returns the range rate, in meters per second, positive when receding.
    pub const fn range_rate(self) -> f64 {
        self.range_rate
    }

    /// Returns the shift, in hertz.
    ///
    /// The value is negative when the terminals are receding.
    pub fn shift(self) -> f64 {
        classical_shift(self.carrier, self.range_rate).unwrap_or(0.0)
    }

    /// Returns the received frequency, in hertz.
    pub fn shifted_frequency(self) -> f64 {
        self.carrier + self.shift()
    }

    /// Returns the shift including the relativistic clock correction, in
    /// hertz.
    ///
    /// The received frequency is the carrier multiplied by the relativistic
    /// factor, so the shift is `f_0 (factor - 1)`.
    ///
    /// This is not the classical shift plus a small correction: the factor
    /// already contains the first order term, and its expansion is
    /// `1 - beta + beta^2 / 2 - ...`, whose leading term is exactly the
    /// classical shift. The classical value is the first order
    /// approximation to this one, and the two differ only by the second and
    /// higher order terms, of order `beta^2`, which for a satellite is a few
    /// parts in ten to the tenth of the carrier.
    ///
    /// The factor is evaluated first and the carrier multiplied afterwards,
    /// because the difference from one is tiny and would lose precision if
    /// the shift were added to the carrier before the correction.
    ///
    /// Returns `None` when the range rate reaches the speed of light.
    pub fn shift_including_relativity(self) -> Option<f64> {
        let factor = relativistic_doppler_factor(self.range_rate)?;
        Some(self.carrier * (factor - 1.0))
    }

    /// Returns the shift in parts per million of the carrier.
    ///
    /// This is a convenient way to compare the shift of links on different
    /// carriers.
    pub fn shift_in_ppm(self) -> f64 {
        self.shift() / self.carrier * 1.0e6
    }

    /// Returns the rate of change of the shift, in hertz per second.
    ///
    /// This is what a Doppler tracker must follow, and it follows from the
    /// range acceleration.
    ///
    /// Returns `None` when the acceleration is not finite.
    pub fn shift_rate(self, range_acceleration: f64) -> Option<f64> {
        if !range_acceleration.is_finite() {
            return None;
        }
        Some(-self.carrier * range_acceleration / SPEED_OF_LIGHT)
    }

    /// Returns `true` when the shift is a redshift, meaning the terminals
    /// are receding and the received frequency is lower.
    pub fn is_receding(self) -> bool {
        self.range_rate > 0.0
    }
}

impl fmt::Display for DopplerShift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} Hz on {} Hz ({:.3} ppm)",
            self.shift(),
            self.carrier,
            self.shift_in_ppm()
        )
    }
}

/// Returns the angle between the line of sight and the velocity, in radians.
///
/// This is the complement of the usual elevation angle within the plane
/// containing both, and it is useful when the range rate is known and the
/// velocity from the orbit is known: the angle follows from the ratio of the
/// two.
///
/// The result lies in `[0, pi]`.
///
/// Returns `None` when the velocity has zero length or either argument is
/// not finite.
pub fn angle_from_range_rate(velocity: Vec3, range_rate: f64) -> Option<f64> {
    if !velocity.is_finite() || !range_rate.is_finite() {
        return None;
    }
    let speed = velocity.norm();
    if speed == 0.0 {
        return None;
    }
    Some((range_rate / speed).clamp(-1.0, 1.0).acos())
}

/// Returns the largest possible shift magnitude for a carrier and a speed,
/// in hertz.
///
/// This is the shift when the motion is entirely along the line of sight,
/// which is the width of the acquisition range a receiver must cover. It is
/// half the total span between the approach and the recession extremes.
///
/// The result is returned as a positive magnitude. Returns `None` when the
/// carrier or speed is not positive and finite.
pub fn maximum_shift(carrier_frequency: f64, speed: f64) -> Option<f64> {
    if !carrier_frequency.is_finite() || carrier_frequency <= 0.0 {
        return None;
    }
    if !speed.is_finite() || speed < 0.0 {
        return None;
    }
    Some(carrier_frequency * speed / SPEED_OF_LIGHT)
}

/// Returns the total span of frequencies a receiver must accept to cover
/// every range rate up to the given speed.
///
/// Approach and recession give shifts of opposite sign, so the span is twice
/// the maximum shift.
///
/// Returns `None` when the carrier or speed is not usable.
pub fn acquisition_span(carrier_frequency: f64, speed: f64) -> Option<f64> {
    maximum_shift(carrier_frequency, speed).map(|shift| 2.0 * shift)
}
