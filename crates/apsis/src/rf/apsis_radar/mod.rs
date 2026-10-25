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
    /// Number of pulses integrated non-coherently.
    pub integrated_pulses: f64,
    /// One-way half-power beamwidth, in degrees.
    pub antenna_beamwidth_deg: f64,
    /// Antenna pointing error from the target line of sight, in degrees.
    pub pointing_error_deg: f64,
    /// Clutter radar cross section in the same resolution cell, in square meters.
    pub clutter_rcs_m2: f64,
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
    /// Received clutter power, in watts.
    pub clutter_power_w: f64,
    /// Signal-to-clutter ratio, in decibels; `None` when clutter is zero.
    pub signal_to_clutter_db: Option<f64>,
    /// Receiver noise power, in watts.
    pub noise_power_w: f64,
    /// Single-pulse signal-to-noise ratio, in decibels.
    pub single_pulse_snr_db: f64,
    /// One-way antenna pattern loss, in decibels.
    pub pattern_loss_db: f64,
    /// Effective antenna gain after pointing loss, in dBi.
    pub effective_gain_dbi: f64,
    /// Integration gain, in decibels.
    pub integration_gain_db: f64,
    /// Integrated signal-to-noise ratio, in decibels.
    pub snr_db: f64,
    /// Maximum detection range for the required SNR, in meters.
    pub max_range_m: f64,
    /// Maximum detection range for the required SNR, in kilometers.
    pub max_range_km: f64,
}


/// Radar search-volume parameters.
///
/// 雷达搜索空域参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarSearchParameters {
    /// Azimuth search span, in degrees.
    pub azimuth_span_deg: f64,
    /// Elevation search span, in degrees.
    pub elevation_span_deg: f64,
    /// Elevation of the search-volume centre, in degrees.
    pub elevation_center_deg: f64,
    /// One-way half-power beamwidth, in degrees.
    pub beamwidth_deg: f64,
    /// Dwell time per beam position, in seconds.
    pub dwell_time_s: f64,
}

/// Radar search-volume results.
///
/// 雷达搜索空域结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarSearchResult {
    /// Beam solid angle, in steradians.
    pub beam_solid_angle_sr: f64,
    /// Search-volume solid angle, in steradians.
    pub search_solid_angle_sr: f64,
    /// Number of beam positions in the search volume.
    pub beam_positions: f64,
    /// Time to cover the search volume once, in seconds.
    pub frame_time_s: f64,
    /// Revisit rate, in hertz.
    pub revisit_rate_hz: f64,
    /// Solid angle searched per second, in steradians per second.
    pub search_rate_sr_s: f64,
}

/// Computes the beam count, frame time, and revisit rate for a search volume.
///
/// The search volume is the spherical rectangle between the elevation limits,
/// and the beam solid angle is approximated by the square of the beamwidth.
///
/// 计算搜索空域的波束数量、帧时间与重访率。
///
/// 搜索空域取仰角上下限之间的球面矩形，波束立体角用波束宽度平方近似。
pub fn radar_search(parameters: RadarSearchParameters) -> Result<RadarSearchResult> {
    let values = [
        parameters.azimuth_span_deg,
        parameters.elevation_span_deg,
        parameters.elevation_center_deg,
        parameters.beamwidth_deg,
        parameters.dwell_time_s,
    ];
    if values.iter().any(|value| !value.is_finite())
        || parameters.azimuth_span_deg <= 0.0
        || parameters.elevation_span_deg <= 0.0
        || parameters.beamwidth_deg <= 0.0
        || parameters.dwell_time_s <= 0.0
        || !(-90.0..=90.0).contains(&parameters.elevation_center_deg)
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "search parameters must be finite and positive",
        ));
    }
    let half_span = (parameters.elevation_span_deg / 2.0).to_radians();
    let center = parameters.elevation_center_deg.to_radians();
    let low = (center - half_span).max(-core::f64::consts::FRAC_PI_2);
    let high = (center + half_span).min(core::f64::consts::FRAC_PI_2);
    if high <= low {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "elevation span is outside the valid range",
        ));
    }
    let azimuth = parameters.azimuth_span_deg.to_radians();
    let beam_solid_angle_sr = parameters.beamwidth_deg.to_radians().powi(2);
    let search_solid_angle_sr = azimuth * (high.sin() - low.sin());
    let beam_positions = (search_solid_angle_sr / beam_solid_angle_sr).ceil();
    let frame_time_s = beam_positions * parameters.dwell_time_s;
    Ok(RadarSearchResult {
        beam_solid_angle_sr,
        search_solid_angle_sr,
        beam_positions,
        frame_time_s,
        revisit_rate_hz: 1.0 / frame_time_s,
        search_rate_sr_s: search_solid_angle_sr / frame_time_s,
    })
}


/// Required SNR for a target detection probability using Albersheim's equation.
///
/// `pd` is the single-pulse detection probability, `pfa` the false-alarm
/// probability, and `pulses` the number of non-coherently integrated pulses.
/// The result is in decibels. Albersheim's equation is an engineering
/// approximation, accurate to a few tenths of a decibel over the usual radar
/// ranges.
///
/// 使用 Albersheim 方程计算达到给定检测概率所需的 SNR。
///
/// `pd` 为单脉冲检测概率，`pfa` 为虚警概率，`pulses` 为非相干积累脉冲数。
/// 结果单位为分贝。Albersheim 方程是工程近似，在常用雷达范围内误差约零点几
/// 分贝。
pub fn albersheim_required_snr_db(pd: f64, pfa: f64, pulses: f64) -> Result<f64> {
    if !pd.is_finite()
        || !pfa.is_finite()
        || !pulses.is_finite()
        || pd <= 0.0
        || pd >= 1.0
        || pfa <= 0.0
        || pfa >= 0.5
        || pulses < 1.0
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "detection probability must be in (0,1), false alarm in (0,0.5), pulses >= 1",
        ));
    }
    let a = (0.62 / pfa).ln();
    let b = (pd / (1.0 - pd)).ln();
    let n_term = 6.2 + 4.54 / (pulses + 0.44).sqrt();
    Ok(-5.0 * pulses.log10()
        + n_term * (a + 0.12 * a * b + 1.7 * b).max(f64::MIN_POSITIVE).log10())
}

/// Estimates detection probability for an integrated SNR.
///
/// Solves Albersheim's equation by bisection. Returns a value in `(0,1)`.
///
/// 根据积累后的 SNR 估算检测概率。
///
/// 用二分法反解 Albersheim 方程，返回 `(0,1)` 内的值。
pub fn detection_probability(snr_db: f64, pfa: f64, pulses: f64) -> Result<f64> {
    if !snr_db.is_finite() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "SNR must be finite",
        ));
    }
    // Validate the fixed parameters once.
    albersheim_required_snr_db(0.5, pfa, pulses)?;
    let mut low = 1.0e-9;
    let mut high = 1.0 - 1.0e-9;
    for _ in 0..80 {
        let middle = 0.5 * (low + high);
        let required = albersheim_required_snr_db(middle, pfa, pulses)?;
        if required <= snr_db {
            low = middle;
        } else {
            high = middle;
        }
    }
    Ok(0.5 * (low + high))
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
        parameters.integrated_pulses,
        parameters.antenna_beamwidth_deg,
        parameters.pointing_error_deg,
        parameters.clutter_rcs_m2,
    ];
    if finite.iter().any(|value| !value.is_finite())
        || parameters.frequency_hz <= 0.0
        || parameters.transmit_power_w <= 0.0
        || parameters.target_rcs_m2 <= 0.0
        || parameters.range_m <= 0.0
        || parameters.bandwidth_hz <= 0.0
        || parameters.system_temperature_k <= 0.0
        || parameters.integrated_pulses < 1.0
        || parameters.antenna_beamwidth_deg <= 0.0
        || parameters.pointing_error_deg < 0.0
        || parameters.clutter_rcs_m2 < 0.0
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "radar parameters must be finite and positive",
        ));
    }
    let wavelength_m = SPEED_OF_LIGHT / parameters.frequency_hz;
    let pattern_loss_db = 12.0
        * (parameters.pointing_error_deg / parameters.antenna_beamwidth_deg).powi(2);
    let effective_gain_dbi = parameters.antenna_gain_dbi - pattern_loss_db;
    let gain = 10.0_f64.powf(effective_gain_dbi / 10.0);
    let loss = 10.0_f64.powf(parameters.system_loss_db / 10.0);
    let numerator = parameters.transmit_power_w
        * gain
        * gain
        * wavelength_m
        * wavelength_m
        * parameters.target_rcs_m2;
    let received_power_w = numerator
        / ((4.0 * core::f64::consts::PI).powi(3) * parameters.range_m.powi(4) * loss);
    let clutter_power_w = received_power_w * parameters.clutter_rcs_m2 / parameters.target_rcs_m2;
    let signal_to_clutter_db = if parameters.clutter_rcs_m2 > 0.0 {
        Some(10.0 * (parameters.target_rcs_m2 / parameters.clutter_rcs_m2).log10())
    } else {
        None
    };
    let noise_power_w =
        BOLTZMANN * parameters.system_temperature_k * parameters.bandwidth_hz;
    let single_pulse_snr_db = 10.0 * (received_power_w / noise_power_w).log10();
    let integration_gain_db = 10.0 * parameters.integrated_pulses.log10();
    let snr_db = single_pulse_snr_db + integration_gain_db;
    let required_snr = 10.0_f64.powf(parameters.required_snr_db / 10.0);
    let max_range_m = (numerator * parameters.integrated_pulses
        / ((4.0 * core::f64::consts::PI).powi(3)
            * loss
            * noise_power_w
            * required_snr))
        .powf(0.25);
    Ok(RadarResult {
        wavelength_m,
        received_power_w,
        clutter_power_w,
        signal_to_clutter_db,
        noise_power_w,
        pattern_loss_db,
        effective_gain_dbi,
        single_pulse_snr_db,
        integration_gain_db,
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
            integrated_pulses: 1.0,
            antenna_beamwidth_deg: 2.0,
            pointing_error_deg: 0.0,
            clutter_rcs_m2: 0.0,
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
    fn clutter_reduces_the_signal_to_clutter_ratio() {
        let clear = radar_budget(parameters()).expect("clear");
        assert_eq!(clear.signal_to_clutter_db, None);
        let cluttered = radar_budget(RadarParameters { clutter_rcs_m2: 10.0, ..parameters() }).expect("clutter");
        assert!((cluttered.signal_to_clutter_db.expect("ratio") + 10.0).abs() < 1e-9);
        assert!((cluttered.clutter_power_w / cluttered.received_power_w - 10.0).abs() < 1e-12);
    }

    #[test]
    fn detection_probability_increases_with_snr() {
        let low = detection_probability(5.0, 1.0e-6, 10.0).expect("low");
        let high = detection_probability(20.0, 1.0e-6, 10.0).expect("high");
        assert!(low > 0.0 && low < 1.0);
        assert!(high > low);
        assert!(high > 0.5);
    }

    #[test]
    fn albersheim_equation_is_monotonic() {
        let easy = albersheim_required_snr_db(0.9, 1.0e-4, 10.0).expect("easy");
        let hard = albersheim_required_snr_db(0.99, 1.0e-8, 10.0).expect("hard");
        assert!(hard > easy);
        assert!(albersheim_required_snr_db(0.9, 0.0, 10.0).is_err());
    }

    #[test]
    fn search_mode_reports_beams_and_frame_time() {
        let result = radar_search(RadarSearchParameters {
            azimuth_span_deg: 360.0,
            elevation_span_deg: 10.0,
            elevation_center_deg: 20.0,
            beamwidth_deg: 2.0,
            dwell_time_s: 0.01,
        }).expect("search");
        assert!(result.beam_positions > 1.0);
        assert!(result.frame_time_s > 0.0);
        assert!((result.frame_time_s - result.beam_positions * 0.01).abs() < 1e-9);
        assert!((result.revisit_rate_hz * result.frame_time_s - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_narrower_beam_needs_more_positions() {
        let wide = radar_search(RadarSearchParameters {
            azimuth_span_deg: 360.0,
            elevation_span_deg: 10.0,
            elevation_center_deg: 20.0,
            beamwidth_deg: 4.0,
            dwell_time_s: 0.01,
        }).expect("wide");
        let narrow = radar_search(RadarSearchParameters {
            azimuth_span_deg: 360.0,
            elevation_span_deg: 10.0,
            elevation_center_deg: 20.0,
            beamwidth_deg: 2.0,
            dwell_time_s: 0.01,
        }).expect("narrow");
        assert!(narrow.beam_positions > wide.beam_positions * 3.5);
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        assert!(radar_budget(RadarParameters { frequency_hz: 0.0, ..parameters() }).is_err());
        assert!(radar_budget(RadarParameters { target_rcs_m2: -1.0, ..parameters() }).is_err());
        assert!(radar_budget(RadarParameters { range_m: f64::NAN, ..parameters() }).is_err());
        assert!(radar_budget(RadarParameters { integrated_pulses: 0.5, ..parameters() }).is_err());
        assert!(radar_budget(RadarParameters { antenna_beamwidth_deg: 0.0, ..parameters() }).is_err());
        assert!(radar_search(RadarSearchParameters {
            azimuth_span_deg: 0.0,
            elevation_span_deg: 10.0,
            elevation_center_deg: 20.0,
            beamwidth_deg: 2.0,
            dwell_time_s: 0.01,
        }).is_err());
    }

    #[test]
    fn pointing_error_reduces_gain_and_range() {
        let on_axis = radar_budget(parameters()).expect("on axis");
        let off_axis = radar_budget(RadarParameters { pointing_error_deg: 2.0, ..parameters() }).expect("off axis");
        assert!((off_axis.pattern_loss_db - 12.0).abs() < 1e-9);
        assert!((off_axis.effective_gain_dbi - (parameters().antenna_gain_dbi - 12.0)).abs() < 1e-9);
        let ratio = off_axis.max_range_km / on_axis.max_range_km;
        assert!((ratio - 10.0_f64.powf(-24.0 / 40.0)).abs() < 1e-9);
    }

    #[test]
    fn pulse_integration_raises_snr_and_range() {
        let single = radar_budget(parameters()).expect("single pulse");
        let integrated = radar_budget(RadarParameters { integrated_pulses: 100.0, ..parameters() }).expect("integrated");
        assert!((integrated.integration_gain_db - 20.0).abs() < 1e-9);
        assert!((integrated.snr_db - single.single_pulse_snr_db - 20.0).abs() < 1e-9);
        let ratio = integrated.max_range_km / single.max_range_km;
        assert!((ratio - 100.0_f64.powf(0.25)).abs() < 1e-9);
    }
}
