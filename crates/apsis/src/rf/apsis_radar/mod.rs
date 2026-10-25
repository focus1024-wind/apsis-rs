//! Monostatic radar range equation.
//!
//! 单站雷达距离方程。

use crate::apsis_core::constant::{BOLTZMANN, SPEED_OF_LIGHT};
use crate::apsis_core::{Error, ErrorKind, Result};

/// Radar and target parameters.
///
/// 雷达与目标参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarParameters {
    /// Carrier frequency, in hertz.
    pub frequency_hz: f64,
    /// Peak transmit power, in watts.
    pub transmit_power_w: f64,
    /// Antenna gain, in decibels relative to isotropic.
    pub antenna_gain_dbi: f64,
    /// Radar cross section, in square meters.
    pub target_rcs_m2: f64,
    /// Target range, in meters.
    pub range_m: f64,
    /// System loss, in decibels.
    pub system_loss_db: f64,
    /// Receiver bandwidth, in hertz.
    pub bandwidth_hz: f64,
    /// System noise temperature, in kelvin.
    pub system_temperature_k: f64,
    /// Required signal-to-noise ratio, in decibels.
    pub required_snr_db: f64,
}

/// Radar budget results.
///
/// 雷达预算结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarResult {
    /// Wavelength, in meters.
    pub wavelength_m: f64,
    /// Received target power, in watts.
    pub received_power_w: f64,
    /// Receiver noise power, in watts.
    pub noise_power_w: f64,
    /// Signal-to-noise ratio, in decibels.
    pub snr_db: f64,
    /// Maximum detection range for the required SNR, in meters.
    pub max_range_m: f64,
    /// Maximum detection range for the required SNR, in kilometers.
    pub max_range_km: f64,
}

/// Computes a monostatic radar budget.
///
/// The radar equation is
///
/// ```text
/// Pr = Pt G^2 lambda^2 sigma / ((4 pi)^3 R^4 L)
/// ```
///
/// and the noise power is `k T B`. The maximum range solves the same
/// equation for `R` at the required SNR.
///
/// 计算单站雷达预算。
///
/// 雷达方程为
///
/// ```text
/// Pr = Pt G^2 lambda^2 sigma / ((4 pi)^3 R^4 L)
/// ```
///
/// 噪声功率为 `k T B`。最大距离由同一方程在要求 SNR 下反解得到。
pub fn radar_budget(parameters: RadarParameters) -> Result<RadarResult> {
    let finite = [
        parameters.frequency_hz,
        parameters.transmit_power_w,
        parameters.antenna_gain_dbi,
        parameters.target_rcs_m2,
        parameters.range_m,
        parameters.system_loss_db,
        parameters.bandwidth_hz,
        parameters.system_temperature_k,
        parameters.required_snr_db,
    ];
    if finite.iter().any(|value| !value.is_finite())
        || parameters.frequency_hz <= 0.0
        || parameters.transmit_power_w <= 0.0
        || parameters.target_rcs_m2 <= 0.0
        || parameters.range_m <= 0.0
        || parameters.bandwidth_hz <= 0.0
        || parameters.system_temperature_k <= 0.0
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "radar parameters must be finite and positive",
        ));
    }
    let wavelength_m = SPEED_OF_LIGHT / parameters.frequency_hz;
    let gain = 10.0_f64.powf(parameters.antenna_gain_dbi / 10.0);
    let loss = 10.0_f64.powf(parameters.system_loss_db / 10.0);
    let numerator = parameters.transmit_power_w
        * gain
        * gain
        * wavelength_m
        * wavelength_m
        * parameters.target_rcs_m2;
    let received_power_w = numerator
        / ((4.0 * core::f64::consts::PI).powi(3) * parameters.range_m.powi(4) * loss);
    let noise_power_w =
        BOLTZMANN * parameters.system_temperature_k * parameters.bandwidth_hz;
    let snr_db = 10.0 * (received_power_w / noise_power_w).log10();
    let required_snr = 10.0_f64.powf(parameters.required_snr_db / 10.0);
    let max_range_m = (numerator
        / ((4.0 * core::f64::consts::PI).powi(3)
            * loss
            * noise_power_w
            * required_snr))
        .powf(0.25);
    Ok(RadarResult {
        wavelength_m,
        received_power_w,
        noise_power_w,
        snr_db,
        max_range_m,
        max_range_km: max_range_m / 1_000.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parameters() -> RadarParameters {
        RadarParameters {
            frequency_hz: 10.0e9,
            transmit_power_w: 1.0e5,
            antenna_gain_dbi: 35.0,
            target_rcs_m2: 1.0,
            range_m: 100_000.0,
            system_loss_db: 6.0,
            bandwidth_hz: 1.0e6,
            system_temperature_k: 500.0,
            required_snr_db: 13.0,
        }
    }

    #[test]
    fn the_radar_budget_is_plausible() {
        let result = radar_budget(parameters()).expect("budget");
        assert!(result.wavelength_m > 0.02 && result.wavelength_m < 0.04);
        assert!(result.received_power_w > 0.0);
        assert!(result.noise_power_w > 0.0);
        assert!(result.snr_db.is_finite());
        assert!(result.max_range_km > 1.0);
        assert!(result.max_range_km < 10_000.0);
    }

    #[test]
    fn snr_falls_by_twelve_db_when_range_doubles() {
        let near = radar_budget(parameters()).expect("near");
        let far = radar_budget(RadarParameters { range_m: 200_000.0, ..parameters() }).expect("far");
        assert!((near.snr_db - far.snr_db - 12.041).abs() < 0.01);
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        assert!(radar_budget(RadarParameters { frequency_hz: 0.0, ..parameters() }).is_err());
        assert!(radar_budget(RadarParameters { target_rcs_m2: -1.0, ..parameters() }).is_err());
        assert!(radar_budget(RadarParameters { range_m: f64::NAN, ..parameters() }).is_err());
    }
}
