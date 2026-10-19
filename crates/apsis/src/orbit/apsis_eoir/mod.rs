//! Electro-optical and infrared sensing.
//!
//! The module models the two halves of an EOIR observation. The optics half
//! turns an aperture, a focal length, and a detector pitch into an
//! instantaneous field of view, a field of view, a ground sample distance,
//! and a diffraction limit. The radiometry half turns a band and a
//! temperature into a Planck radiance, and the difference between a target
//! and its background into a contrast radiance.
//!
//! EOIR 光电与红外探测。
//!
//! 本模块建模 EOIR 观测的两半。光学半部把口径、焦距与探测器像元间距换算为
//! 瞬时视场、总视场、地面采样距离与衍射极限。辐射半部把波段与温度换算为
//! 普朗克辐亮度，并把目标与背景之差换算为对比辐亮度。

use crate::apsis_core::{Error, ErrorKind, Result};

/// The Planck constant, in joule seconds.
///
/// 普朗克常数，单位为焦耳秒。
pub const PLANCK_CONSTANT: f64 = 6.626_070_15e-34;

/// The speed of light in vacuum, in metres per second.
///
/// 真空光速，单位为米每秒。
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// The Boltzmann constant, in joules per kelvin.
///
/// 玻尔兹曼常数，单位为焦耳每开尔文。
pub const BOLTZMANN_CONSTANT: f64 = 1.380_649e-23;

/// The Stefan-Boltzmann constant, in watts per square metre kelvin to the
/// fourth.
///
/// 斯特藩-玻尔兹曼常数，单位为瓦特每平方米四次方开尔文。
pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

/// The Wien displacement constant, in metre kelvin.
///
/// 维恩位移常数，单位为米开尔文。
pub const WIEN_DISPLACEMENT: f64 = 2.897_771_955e-3;

/// A spectral band, given by its wavelength limits.
///
/// 由其波长上下限给出的光谱波段。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpectralBand {
    /// Shortest wavelength in micrometres.
    ///
    /// 最短波长，单位为微米。
    pub min_micrometres: f64,
    /// Longest wavelength in micrometres.
    ///
    /// 最长波长，单位为微米。
    pub max_micrometres: f64,
}

impl SpectralBand {
    /// Creates a band from its limits.
    ///
    /// Returns an error unless `0 < min < max` and both are finite.
    ///
    /// 由上下限创建波段。
    ///
    /// 除非 `0 < min < max` 且二者有限，否则返回错误。
    pub fn new(min_micrometres: f64, max_micrometres: f64) -> Result<Self> {
        if !min_micrometres.is_finite()
            || !max_micrometres.is_finite()
            || min_micrometres <= 0.0
            || max_micrometres <= min_micrometres
        {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "band limits must satisfy 0 < min < max".to_string(),
            ));
        }
        Ok(Self {
            min_micrometres,
            max_micrometres,
        })
    }

    /// The visible band, 0.4 to 0.7 micrometres.
    ///
    /// 可见光波段，0.4 至 0.7 微米。
    pub const VISIBLE: Self = Self {
        min_micrometres: 0.4,
        max_micrometres: 0.7,
    };

    /// The near infrared band, 0.7 to 1.0 micrometres.
    ///
    /// 近红外波段，0.7 至 1.0 微米。
    pub const NIR: Self = Self {
        min_micrometres: 0.7,
        max_micrometres: 1.0,
    };

    /// The short wave infrared band, 1.0 to 3.0 micrometres.
    ///
    /// 短波红外波段，1.0 至 3.0 微米。
    pub const SWIR: Self = Self {
        min_micrometres: 1.0,
        max_micrometres: 3.0,
    };

    /// The medium wave infrared band, 3.0 to 5.0 micrometres.
    ///
    /// 中波红外波段，3.0 至 5.0 微米。
    pub const MWIR: Self = Self {
        min_micrometres: 3.0,
        max_micrometres: 5.0,
    };

    /// The long wave infrared band, 8.0 to 12.0 micrometres.
    ///
    /// 长波红外波段，8.0 至 12.0 微米。
    pub const LWIR: Self = Self {
        min_micrometres: 8.0,
        max_micrometres: 12.0,
    };

    /// Returns the band width in micrometres.
    ///
    /// 返回波段宽度，单位为微米。
    #[must_use]
    pub fn width_micrometres(self) -> f64 {
        self.max_micrometres - self.min_micrometres
    }

    /// Returns the centre wavelength in micrometres.
    ///
    /// 返回中心波长，单位为微米。
    #[must_use]
    pub fn centre_micrometres(self) -> f64 {
        f64::midpoint(self.min_micrometres, self.max_micrometres)
    }
}

/// Returns the Planck spectral radiance at a wavelength and temperature.
///
/// The result is in watts per square metre steradian micrometre. Returns an
/// error when the wavelength or temperature is not positive and finite.
///
/// 返回某波长与温度下的普朗克光谱辐亮度。
///
/// 结果单位为瓦特每平方米球面度微米。波长或温度非正且有限时返回错误。
pub fn planck_radiance(wavelength_micrometres: f64, temperature_kelvin: f64) -> Result<f64> {
    if !wavelength_micrometres.is_finite() || wavelength_micrometres <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "wavelength must be positive and finite".to_string(),
        ));
    }
    if !temperature_kelvin.is_finite() || temperature_kelvin <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "temperature must be positive and finite".to_string(),
        ));
    }
    let wavelength = wavelength_micrometres * 1e-6;
    let exponent =
        PLANCK_CONSTANT * SPEED_OF_LIGHT / (wavelength * BOLTZMANN_CONSTANT * temperature_kelvin);
    if exponent > 700.0 {
        return Ok(0.0);
    }
    let numerator = 2.0 * PLANCK_CONSTANT * SPEED_OF_LIGHT * SPEED_OF_LIGHT / wavelength.powi(5);
    let per_metre = numerator / exponent.exp_m1();
    Ok(per_metre * 1e-6)
}

/// Returns the Planck radiance integrated over a band.
///
/// The integration uses Simpson's rule over 512 intervals, which resolves
/// the smooth Planck curve to well below a percent for any band above
/// 0.1 micrometres. The result is in watts per square metre steradian.
///
/// 返回某波段内的普朗克辐亮度积分。
///
/// 积分在 512 个区间上使用辛普森法则，对 0.1 微米以上的任何波段都能把光滑的
/// 普朗克曲线解析到远低于百分之一的误差。结果单位为瓦特每平方米球面度。
pub fn band_radiance(band: SpectralBand, temperature_kelvin: f64) -> Result<f64> {
    if !temperature_kelvin.is_finite() || temperature_kelvin <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "temperature must be positive and finite".to_string(),
        ));
    }
    const INTERVALS: usize = 512;
    let start = band.min_micrometres;
    let end = band.max_micrometres;
    let step = (end - start) / INTERVALS as f64;
    let mut sum =
        planck_radiance(start, temperature_kelvin)? + planck_radiance(end, temperature_kelvin)?;
    for index in 1..INTERVALS {
        let weight = if index % 2 == 0 { 2.0 } else { 4.0 };
        sum += weight * planck_radiance(start + index as f64 * step, temperature_kelvin)?;
    }
    Ok(sum * step / 3.0)
}

/// Returns the total blackbody radiance at a temperature.
///
/// This is `sigma T^4 / pi`, the Stefan-Boltzmann law divided by pi because
/// radiance is per steradian. The result is in watts per square metre
/// steradian.
///
/// 返回某温度下的黑体总辐亮度。
///
/// 即 `sigma T^4 / pi`，斯特藩-玻尔兹曼定律除以 pi，因为辐亮度按球面度计。
/// 结果单位为瓦特每平方米球面度。
pub fn total_radiance(temperature_kelvin: f64) -> Result<f64> {
    if !temperature_kelvin.is_finite() || temperature_kelvin <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "temperature must be positive and finite".to_string(),
        ));
    }
    Ok(STEFAN_BOLTZMANN * temperature_kelvin.powi(4) / core::f64::consts::PI)
}

/// Returns the fraction of the total radiance that falls inside a band.
///
/// 返回落在某波段内的总辐亮度占比。
pub fn band_fraction(band: SpectralBand, temperature_kelvin: f64) -> Result<f64> {
    let band_value = band_radiance(band, temperature_kelvin)?;
    let total = total_radiance(temperature_kelvin)?;
    Ok(band_value / total)
}

/// The optical and detector geometry of an EOIR sensor.
///
/// EOIR 传感器的光学与探测器几何。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EoirSensor {
    /// Entrance aperture diameter in metres.
    ///
    /// 入瞳直径，单位为米。
    pub aperture_m: f64,
    /// Effective focal length in metres.
    ///
    /// 有效焦距，单位为米。
    pub focal_length_m: f64,
    /// Detector pixel pitch in metres.
    ///
    /// 探测器像元间距，单位为米。
    pub pixel_pitch_m: f64,
    /// Number of detector columns.
    ///
    /// 探测器列数。
    pub horizontal_pixels: u32,
    /// Number of detector rows.
    ///
    /// 探测器行数。
    pub vertical_pixels: u32,
}

impl EoirSensor {
    /// Creates a sensor.
    ///
    /// Returns an error unless every length is positive and finite and both
    /// pixel counts are non-zero.
    ///
    /// 创建传感器。
    ///
    /// 除非每个长度均为正且有限、两个像元数均非零，否则返回错误。
    pub fn new(
        aperture_m: f64,
        focal_length_m: f64,
        pixel_pitch_m: f64,
        horizontal_pixels: u32,
        vertical_pixels: u32,
    ) -> Result<Self> {
        if !aperture_m.is_finite()
            || aperture_m <= 0.0
            || !focal_length_m.is_finite()
            || focal_length_m <= 0.0
            || !pixel_pitch_m.is_finite()
            || pixel_pitch_m <= 0.0
            || horizontal_pixels == 0
            || vertical_pixels == 0
        {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "sensor geometry must be positive and finite".to_string(),
            ));
        }
        Ok(Self {
            aperture_m,
            focal_length_m,
            pixel_pitch_m,
            horizontal_pixels,
            vertical_pixels,
        })
    }

    /// Returns the instantaneous field of view in radians.
    ///
    /// This is the pixel pitch divided by the focal length, the angle one
    /// detector element subtends at the entrance pupil.
    ///
    /// 返回瞬时视场，单位为弧度。
    ///
    /// 即像元间距除以焦距，是一个探测元在入瞳处张开的角。
    #[must_use]
    pub fn ifov_radians(self) -> f64 {
        self.pixel_pitch_m / self.focal_length_m
    }

    /// Returns the horizontal field of view in radians.
    ///
    /// 返回水平视场，单位为弧度。
    #[must_use]
    pub fn fov_radians(self) -> f64 {
        2.0 * (self.horizontal_pixels as f64 * self.pixel_pitch_m / (2.0 * self.focal_length_m))
            .atan()
    }

    /// Returns the horizontal field of view in degrees.
    ///
    /// 返回水平视场，单位为度。
    #[must_use]
    pub fn fov_degrees(self) -> f64 {
        self.fov_radians().to_degrees()
    }

    /// Returns the diffraction-limited angular resolution at a wavelength.
    ///
    /// This is the Rayleigh criterion `1.22 lambda / D`, in radians.
    ///
    /// 返回某波长下的衍射极限角分辨率。
    ///
    /// 即瑞利判据 `1.22 lambda / D`，单位为弧度。
    pub fn diffraction_limit_radians(self, wavelength_micrometres: f64) -> Result<f64> {
        if !wavelength_micrometres.is_finite() || wavelength_micrometres <= 0.0 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "wavelength must be positive and finite".to_string(),
            ));
        }
        Ok(1.22 * wavelength_micrometres * 1e-6 / self.aperture_m)
    }

    /// Returns the Airy disk diameter on the focal plane, in metres.
    ///
    /// 返回焦平面上的艾里斑直径，单位为米。
    pub fn airy_diameter_m(self, wavelength_micrometres: f64) -> Result<f64> {
        let limit = self.diffraction_limit_radians(wavelength_micrometres)?;
        Ok(2.0 * limit * self.focal_length_m)
    }

    /// Returns the ground sample distance at a range, in metres.
    ///
    /// 返回某距离处的地面采样距离，单位为米。
    pub fn gsd_m(self, range_m: f64) -> Result<f64> {
        if !range_m.is_finite() || range_m <= 0.0 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "range must be positive and finite".to_string(),
            ));
        }
        Ok(self.ifov_radians() * range_m)
    }

    /// Returns the ground footprint width and height at a range, in metres.
    ///
    /// 返回某距离处的地面覆盖宽高，单位为米。
    pub fn footprint_m(self, range_m: f64) -> Result<(f64, f64)> {
        let gsd = self.gsd_m(range_m)?;
        Ok((
            gsd * f64::from(self.horizontal_pixels),
            gsd * f64::from(self.vertical_pixels),
        ))
    }
}

/// The result of an EOIR observation analysis.
///
/// EOIR 观测分析的结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EoirAnalysis {
    /// Optical geometry of the sensor.
    ///
    /// 传感器的光学几何。
    pub sensor: EoirSensor,
    /// Spectral band.
    ///
    /// 光谱波段。
    pub band: SpectralBand,
    /// Range to the target in metres.
    ///
    /// 到目标的距离，单位为米。
    pub range_m: f64,
    /// Ground sample distance at the range, in metres.
    ///
    /// 该距离处的地面采样距离，单位为米。
    pub gsd_m: f64,
    /// Target size in metres.
    ///
    /// 目标尺寸，单位为米。
    pub target_size_m: f64,
    /// Number of pixels across the target.
    ///
    /// 目标横向所占像元数。
    pub target_pixels: f64,
    /// Greatest range at which the target still spans the required pixels.
    ///
    /// 目标仍能占满所需像元数的最大距离。
    pub max_range_m: f64,
    /// Target in-band radiance in watts per square metre steradian.
    ///
    /// 目标的带内辐亮度，单位为瓦特每平方米球面度。
    pub target_radiance: f64,
    /// Background in-band radiance.
    ///
    /// 背景的带内辐亮度。
    pub background_radiance: f64,
    /// Contrast radiance after emissivity.
    ///
    /// 乘上发射率后的对比辐亮度。
    pub contrast_radiance: f64,
    /// Diffraction limit at the band centre, in radians.
    ///
    /// 波段中心处的衍射极限，单位为弧度。
    pub diffraction_limit_radians: f64,
}

/// Analyzes an EOIR observation.
///
/// Returns an error when the sensor, band, range, temperatures, emissivity,
/// target size, or pixel requirement is not usable.
///
/// 分析一次 EOIR 观测。
///
/// 传感器、波段、距离、温度、发射率、目标尺寸或像元要求不可用时返回错误。
#[allow(clippy::too_many_arguments)]
pub fn analyze(
    sensor: EoirSensor,
    band: SpectralBand,
    range_m: f64,
    target_temperature_kelvin: f64,
    background_temperature_kelvin: f64,
    emissivity: f64,
    target_size_m: f64,
    required_pixels: f64,
) -> Result<EoirAnalysis> {
    if !range_m.is_finite() || range_m <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "range must be positive and finite".to_string(),
        ));
    }
    if !target_size_m.is_finite() || target_size_m <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "target size must be positive and finite".to_string(),
        ));
    }
    if !emissivity.is_finite() || !(0.0..=1.0).contains(&emissivity) {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "emissivity must be within 0..=1".to_string(),
        ));
    }
    if !required_pixels.is_finite() || required_pixels <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "required pixels must be positive and finite".to_string(),
        ));
    }
    let gsd = sensor.gsd_m(range_m)?;
    let target_radiance = band_radiance(band, target_temperature_kelvin)?;
    let background_radiance = band_radiance(band, background_temperature_kelvin)?;
    Ok(EoirAnalysis {
        sensor,
        band,
        range_m,
        gsd_m: gsd,
        target_size_m,
        target_pixels: target_size_m / gsd,
        max_range_m: target_size_m / (required_pixels * sensor.ifov_radians()),
        target_radiance,
        background_radiance,
        contrast_radiance: emissivity * (target_radiance - background_radiance),
        diffraction_limit_radians: sensor.diffraction_limit_radians(band.centre_micrometres())?,
    })
}
