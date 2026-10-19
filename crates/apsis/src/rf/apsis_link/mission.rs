//! Link budgets for a radio link.
//!
//! The module wraps the validated apsis link chain in the terms a mission
//! user supplies: a frequency, a transmit power and antenna gain, the losses
//! on each side, a system noise temperature, a receiver bandwidth, a data
//! rate, and a required energy per bit over noise density. Every loss is a
//! positive decibel value that is subtracted, following the usual link budget
//! convention.
//!
//! 无线电链路的链路预算。
//!
//! 本模块把已验证的 apsis 链路链包装为任务用户提供的量：频率、发射功率与
//! 天线增益、两侧损耗、系统噪声温度、接收机带宽、数据速率，以及要求的每比特
//! 能量与噪声密度之比。每项损耗都是被减去的正分贝值，遵循通常的链路预算约定。

use crate::apsis_link::{
    Decibel, Gain, LinkBudget, LinkGeometry, NoiseTemperature, Power, gain_to_temperature,
    link_margin,
};

use crate::apsis_core::{Error, ErrorKind, Result};

/// The inputs of a link budget.
///
/// 链路预算的输入。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkParameters {
    /// Carrier frequency in hertz.
    ///
    /// 载波频率，单位为赫兹。
    pub frequency_hz: f64,
    /// Power delivered to the transmit antenna, in watts.
    ///
    /// 送到发射天线的功率，单位为瓦特。
    pub transmit_power_watts: f64,
    /// Transmit antenna gain in dBi.
    ///
    /// 发射天线增益，单位为 dBi。
    pub transmit_gain_dbi: f64,
    /// Loss between the transmitter and the antenna, in decibels.
    ///
    /// 发射机与天线之间的损耗，单位为分贝。
    pub transmit_loss_db: f64,
    /// Receive antenna gain in dBi.
    ///
    /// 接收天线增益，单位为 dBi。
    pub receive_gain_dbi: f64,
    /// Loss between the antenna and the receiver, in decibels.
    ///
    /// 天线与接收机之间的损耗，单位为分贝。
    pub receive_loss_db: f64,
    /// System noise temperature referred to the receiver input, in kelvin.
    ///
    /// 折算到接收机输入端的系统噪声温度，单位为开尔文。
    pub system_temperature_kelvin: f64,
    /// Receiver noise bandwidth in hertz.
    ///
    /// 接收机噪声带宽，单位为赫兹。
    pub bandwidth_hz: f64,
    /// Data rate in bits per second.
    ///
    /// 数据速率，单位为比特每秒。
    pub data_rate_bps: f64,
    /// Required energy per bit over noise density, in decibels.
    ///
    /// 要求的每比特能量与噪声密度之比，单位为分贝。
    pub required_ebno_db: f64,
}

impl Default for LinkParameters {
    fn default() -> Self {
        Self {
            frequency_hz: 12.0e9,
            transmit_power_watts: 10.0,
            transmit_gain_dbi: 30.0,
            transmit_loss_db: 1.0,
            receive_gain_dbi: 35.0,
            receive_loss_db: 0.5,
            system_temperature_kelvin: 150.0,
            bandwidth_hz: 36.0e6,
            data_rate_bps: 10.0e6,
            required_ebno_db: 4.0,
        }
    }
}

/// The figures produced by a link budget.
///
/// 链路预算所得的各数字。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkResult {
    /// Distance between the terminals in meters.
    ///
    /// 两端之间的距离，单位为米。
    pub distance_meters: f64,
    /// Free space path loss in decibels.
    ///
    /// 自由空间路径损耗，单位为分贝。
    pub path_loss_db: f64,
    /// Equivalent isotropically radiated power in dBW.
    ///
    /// 等效全向辐射功率，单位为 dBW。
    pub eirp_dbw: f64,
    /// Carrier power at the receiver input in dBW.
    ///
    /// 接收机输入端的载波功率，单位为 dBW。
    pub received_power_dbw: f64,
    /// Noise power density in dBW per hertz.
    ///
    /// 噪声功率密度，单位为 dBW 每赫兹。
    pub noise_density_dbw_per_hz: f64,
    /// Carrier to noise density ratio in decibels.
    ///
    /// 载噪密度比，单位为分贝。
    pub carrier_to_noise_density_db: f64,
    /// Carrier to noise ratio in the receiver bandwidth, in decibels.
    ///
    /// 接收机带宽内的载噪比，单位为分贝。
    pub carrier_to_noise_db: f64,
    /// Energy per bit over noise density in decibels.
    ///
    /// 每比特能量与噪声密度之比，单位为分贝。
    pub ebno_db: f64,
    /// Link margin against the required Eb/N0, in decibels.
    ///
    /// 相对要求 Eb/N0 的链路余量，单位为分贝。
    pub margin_db: f64,
    /// Whether the margin is not negative.
    ///
    /// 余量非负时为真。
    pub closes: bool,
    /// Receive gain over system temperature, in decibels per kelvin.
    ///
    /// 接收增益与系统温度之比，单位为分贝每开尔文。
    pub gain_to_temperature_db: f64,
}

/// Computes the link budget for a distance and a parameter set.
///
/// Returns an error when a parameter is not usable or when the link chain
/// cannot produce a figure, which happens for a non-positive power, gain, or
/// temperature.
///
/// 按距离与参数集计算链路预算。
///
/// 参数不可用，或链路链无法给出数字（功率、增益或温度非正）时返回错误。
pub fn link_budget(distance_meters: f64, parameters: &LinkParameters) -> Result<LinkResult> {
    validate(distance_meters, parameters)?;
    let budget = LinkBudget {
        transmit_power: Power::from_watts(parameters.transmit_power_watts),
        transmit_gain: Gain::from_dbi(parameters.transmit_gain_dbi).ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidInput,
                "transmit gain is not a finite dBi value".to_string(),
            )
        })?,
        transmit_loss: Decibel::from_db(parameters.transmit_loss_db),
        receive_gain: Gain::from_dbi(parameters.receive_gain_dbi).ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidInput,
                "receive gain is not a finite dBi value".to_string(),
            )
        })?,
        receive_loss: Decibel::from_db(parameters.receive_loss_db),
        system_temperature: NoiseTemperature::from_kelvin(parameters.system_temperature_kelvin),
        bandwidth: parameters.bandwidth_hz,
        geometry: LinkGeometry {
            distance: distance_meters,
            frequency: parameters.frequency_hz,
        },
    };
    let path_loss = budget.geometry.path_loss().ok_or_else(|| {
        Error::with_message(
            ErrorKind::InvalidInput,
            "path loss is undefined for this geometry".to_string(),
        )
    })?;
    let eirp = budget.eirp().ok_or_else(|| {
        Error::with_message(ErrorKind::InvalidInput, "EIRP is undefined".to_string())
    })?;
    let carrier = budget.received_power().ok_or_else(|| {
        Error::with_message(
            ErrorKind::InvalidInput,
            "carrier power is undefined".to_string(),
        )
    })?;
    let density = budget.noise_density().ok_or_else(|| {
        Error::with_message(
            ErrorKind::InvalidInput,
            "noise density is undefined".to_string(),
        )
    })?;
    let carrier_to_noise_density = budget.carrier_to_noise_density().ok_or_else(|| {
        Error::with_message(ErrorKind::InvalidInput, "C/N0 is undefined".to_string())
    })?;
    let carrier_to_noise = budget.carrier_to_noise().ok_or_else(|| {
        Error::with_message(ErrorKind::InvalidInput, "C/N is undefined".to_string())
    })?;
    let ebno = budget
        .energy_per_bit_over_noise(parameters.data_rate_bps)
        .ok_or_else(|| {
            Error::with_message(ErrorKind::InvalidInput, "Eb/N0 is undefined".to_string())
        })?;
    let margin = link_margin(ebno, Decibel::from_db(parameters.required_ebno_db));
    let gain_to_temperature = gain_to_temperature(
        Gain::from_dbi(parameters.receive_gain_dbi).expect("validated"),
        NoiseTemperature::from_kelvin(parameters.system_temperature_kelvin),
    )
    .ok_or_else(|| Error::with_message(ErrorKind::InvalidInput, "G/T is undefined".to_string()))?;
    Ok(LinkResult {
        distance_meters,
        path_loss_db: path_loss.db(),
        eirp_dbw: eirp.dbw().ok_or_else(|| {
            Error::with_message(ErrorKind::InvalidInput, "EIRP is not positive".to_string())
        })?,
        received_power_dbw: carrier.dbw().ok_or_else(|| {
            Error::with_message(
                ErrorKind::InvalidInput,
                "carrier power is not positive".to_string(),
            )
        })?,
        noise_density_dbw_per_hz: 10.0 * density.log10(),
        carrier_to_noise_density_db: carrier_to_noise_density.db(),
        carrier_to_noise_db: carrier_to_noise.db(),
        ebno_db: ebno.db(),
        margin_db: margin.margin().db(),
        closes: margin.closes(),
        gain_to_temperature_db: 10.0 * gain_to_temperature.log10(),
    })
}

fn validate(distance_meters: f64, parameters: &LinkParameters) -> Result<()> {
    if !distance_meters.is_finite() || distance_meters <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "distance must be a positive finite number of meters".to_string(),
        ));
    }
    if !parameters.frequency_hz.is_finite() || parameters.frequency_hz <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "frequency must be a positive finite number of hertz".to_string(),
        ));
    }
    if !parameters.transmit_power_watts.is_finite() || parameters.transmit_power_watts <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "transmit power must be a positive finite number of watts".to_string(),
        ));
    }
    if !parameters.transmit_loss_db.is_finite() || !parameters.receive_loss_db.is_finite() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "losses must be finite decibel values".to_string(),
        ));
    }
    if parameters.transmit_loss_db < 0.0 || parameters.receive_loss_db < 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "losses must be entered as positive decibel values".to_string(),
        ));
    }
    if !parameters.system_temperature_kelvin.is_finite()
        || parameters.system_temperature_kelvin <= 0.0
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "system temperature must be a positive finite number of kelvin".to_string(),
        ));
    }
    if !parameters.bandwidth_hz.is_finite() || parameters.bandwidth_hz <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "bandwidth must be a positive finite number of hertz".to_string(),
        ));
    }
    if !parameters.data_rate_bps.is_finite() || parameters.data_rate_bps <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "data rate must be a positive finite number of bits per second".to_string(),
        ));
    }
    if !parameters.required_ebno_db.is_finite() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "required Eb/N0 must be a finite decibel value".to_string(),
        ));
    }
    Ok(())
}
