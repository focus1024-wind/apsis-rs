//! Aircraft performance in the standard atmosphere.
//!
//! The module computes the state of the standard atmosphere at a geometric
//! altitude from the ITU-R P.835 profile, then the level-flight performance
//! of a simple aircraft model: lift coefficient from the wing loading, drag
//! from the parabolic polar, thrust required to hold the speed, climb rate
//! from the excess thrust, and fuel flow from the thrust-specific fuel
//! consumption. The Breguet range and endurance follow from the same model.
//!
//! 标准大气中的航空器性能。
//!
//! 本模块先由 ITU-R P.835 廓线计算某几何高度处的标准大气状态，再计算简单
//! 航空器模型的平飞性能：由翼载得到升力系数、由抛物线极曲线得到阻力、维持
//! 速度所需的推力、由剩余推力得到爬升率，以及由推力比油耗得到燃油流量。
//! 布雷盖航程与航时由同一模型得出。

use crate::apsis_atmosphere::{standard_pressure, standard_temperature};

use crate::apsis_core::{Error, ErrorKind, Result};

/// The specific gas constant of dry air, in joules per kilogram kelvin.
///
/// 干空气比气体常数，单位为焦耳每千克开尔文。
pub const DRY_AIR_GAS_CONSTANT: f64 = 287.052_874;

/// The standard gravitational acceleration, in metres per second squared.
///
/// 标准重力加速度，单位为米每二次方秒。
pub const STANDARD_GRAVITY: f64 = 9.806_65;

/// The ratio of specific heats of air.
///
/// 空气的比热比。
pub const AIR_HEAT_RATIO: f64 = 1.4;

/// The highest geometric altitude the aircraft model accepts, in metres.
///
/// 航空器模型接受的最高几何高度，单位为米。
pub const MAX_ALTITUDE_METERS: f64 = 20_000.0;

/// The state of the standard atmosphere at one altitude.
///
/// 某高度处标准大气的状态。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtmosphereState {
    /// Geometric altitude in metres.
    ///
    /// 几何高度，单位为米。
    pub altitude_meters: f64,
    /// Temperature in kelvin.
    ///
    /// 温度，单位为开尔文。
    pub temperature_kelvin: f64,
    /// Pressure in pascals.
    ///
    /// 气压，单位为帕斯卡。
    pub pressure_pascals: f64,
    /// Density in kilograms per cubic metre.
    ///
    /// 密度，单位为千克每立方米。
    pub density_kg_m3: f64,
    /// Speed of sound in metres per second.
    ///
    /// 声速，单位为米每秒。
    pub speed_of_sound_m_s: f64,
}

impl AtmosphereState {
    /// Returns the dynamic pressure at a true airspeed, in pascals.
    ///
    /// 返回某真空速下的动压，单位为帕斯卡。
    #[must_use]
    pub fn dynamic_pressure(self, speed_m_s: f64) -> f64 {
        0.5 * self.density_kg_m3 * speed_m_s * speed_m_s
    }

    /// Returns the Mach number at a true airspeed.
    ///
    /// 返回某真空速下的马赫数。
    #[must_use]
    pub fn mach(self, speed_m_s: f64) -> f64 {
        speed_m_s / self.speed_of_sound_m_s
    }
}

/// Returns the standard atmosphere at a geometric altitude.
///
/// Returns an error when the altitude is not finite or lies outside
/// `0..=20_000` metres.
///
/// 返回某几何高度处的标准大气。
///
/// 高度非有限或超出 `0..=20000` 米时返回错误。
pub fn atmosphere(altitude_meters: f64) -> Result<AtmosphereState> {
    if !altitude_meters.is_finite() || !(0.0..=MAX_ALTITUDE_METERS).contains(&altitude_meters) {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "altitude must be finite and within 0..=20000 meters".to_string(),
        ));
    }
    let altitude_km = altitude_meters / 1_000.0;
    let temperature_kelvin = standard_temperature(altitude_km);
    let pressure_pascals = standard_pressure(altitude_km) * 100.0;
    if temperature_kelvin <= 0.0 || pressure_pascals <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "standard atmosphere is not defined at this altitude".to_string(),
        ));
    }
    let density_kg_m3 = pressure_pascals / (DRY_AIR_GAS_CONSTANT * temperature_kelvin);
    let speed_of_sound_m_s = (AIR_HEAT_RATIO * DRY_AIR_GAS_CONSTANT * temperature_kelvin).sqrt();
    Ok(AtmosphereState {
        altitude_meters,
        temperature_kelvin,
        pressure_pascals,
        density_kg_m3,
        speed_of_sound_m_s,
    })
}

/// A simple aircraft model used for level-flight performance.
///
/// 用于平飞性能的简单航空器模型。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aircraft {
    /// Mass in kilograms.
    ///
    /// 质量，单位为千克。
    pub mass_kg: f64,
    /// Wing reference area in square metres.
    ///
    /// 机翼参考面积，单位为平方米。
    pub wing_area_m2: f64,
    /// Zero-lift drag coefficient.
    ///
    /// 零升阻力系数。
    pub zero_lift_drag: f64,
    /// Oswald span efficiency factor.
    ///
    /// 奥斯瓦尔德展向效率因子。
    pub oswald_efficiency: f64,
    /// Wing aspect ratio.
    ///
    /// 机翼展弦比。
    pub aspect_ratio: f64,
    /// Maximum thrust available at the current condition, in newtons.
    ///
    /// 当前条件下可用的最大推力，单位为牛顿。
    pub max_thrust_n: f64,
    /// Thrust-specific fuel consumption, in kilograms per newton second.
    ///
    /// 推力比油耗，单位为千克每牛顿秒。
    pub tsfc_kg_per_n_s: f64,
}

impl Default for Aircraft {
    fn default() -> Self {
        Self {
            mass_kg: 1_000.0,
            wing_area_m2: 16.0,
            zero_lift_drag: 0.03,
            oswald_efficiency: 0.8,
            aspect_ratio: 7.0,
            max_thrust_n: 3_000.0,
            tsfc_kg_per_n_s: 1.5e-5,
        }
    }
}

impl Aircraft {
    /// Returns the weight in newtons.
    ///
    /// 返回重力，单位为牛顿。
    #[must_use]
    pub fn weight_n(self) -> f64 {
        self.mass_kg * STANDARD_GRAVITY
    }

    /// Returns the wing loading in newtons per square metre.
    ///
    /// 返回翼载，单位为牛顿每平方米。
    #[must_use]
    pub fn wing_loading_n_m2(self) -> f64 {
        if self.wing_area_m2 <= 0.0 {
            0.0
        } else {
            self.weight_n() / self.wing_area_m2
        }
    }

    /// Returns true when every parameter is usable.
    ///
    /// 各参数均可用时返回真。
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.mass_kg.is_finite()
            && self.mass_kg > 0.0
            && self.wing_area_m2.is_finite()
            && self.wing_area_m2 > 0.0
            && self.zero_lift_drag.is_finite()
            && self.zero_lift_drag > 0.0
            && self.oswald_efficiency.is_finite()
            && self.oswald_efficiency > 0.0
            && self.oswald_efficiency <= 1.0
            && self.aspect_ratio.is_finite()
            && self.aspect_ratio > 0.0
            && self.max_thrust_n.is_finite()
            && self.max_thrust_n >= 0.0
            && self.tsfc_kg_per_n_s.is_finite()
            && self.tsfc_kg_per_n_s >= 0.0
    }
}

/// The result of a level-flight performance calculation.
///
/// 平飞性能计算的结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelFlight {
    /// Atmosphere at the flight altitude.
    ///
    /// 飞行高度处的大气。
    pub atmosphere: AtmosphereState,
    /// True airspeed in metres per second.
    ///
    /// 真空速，单位为米每秒。
    pub speed_m_s: f64,
    /// Mach number.
    ///
    /// 马赫数。
    pub mach: f64,
    /// Lift coefficient required to balance the weight.
    ///
    /// 平衡重力所需的升力系数。
    pub lift_coefficient: f64,
    /// Drag coefficient from the parabolic polar.
    ///
    /// 由抛物线极曲线得到的阻力系数。
    pub drag_coefficient: f64,
    /// Drag force in newtons.
    ///
    /// 阻力，单位为牛顿。
    pub drag_n: f64,
    /// Thrust required to hold the speed, in newtons.
    ///
    /// 维持速度所需的推力，单位为牛顿。
    pub thrust_required_n: f64,
    /// Thrust actually available, in newtons.
    ///
    /// 实际可用推力，单位为牛顿。
    pub thrust_available_n: f64,
    /// Power required to overcome drag, in watts.
    ///
    /// 克服阻力所需的功率，单位为瓦特。
    pub power_required_w: f64,
    /// Rate of climb from the excess thrust, in metres per second.
    ///
    /// 由剩余推力得到的爬升率，单位为米每秒。
    pub climb_rate_m_s: f64,
    /// Fuel flow in kilograms per second.
    ///
    /// 燃油流量，单位为千克每秒。
    pub fuel_flow_kg_s: f64,
    /// Lift-to-drag ratio.
    ///
    /// 升阻比。
    pub lift_to_drag: f64,
}

/// Computes level-flight performance at an altitude, speed, and thrust.
///
/// Returns an error when the aircraft parameters, altitude, speed, or thrust
/// are not usable.
///
/// 计算某高度、速度与推力下的平飞性能。
///
/// 航空器参数、高度、速度或推力不可用时返回错误。
pub fn level_flight(
    aircraft: &Aircraft,
    altitude_meters: f64,
    speed_m_s: f64,
    thrust_available_n: f64,
) -> Result<LevelFlight> {
    if !aircraft.is_valid() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "aircraft parameters must be positive and finite".to_string(),
        ));
    }
    if !speed_m_s.is_finite() || speed_m_s <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "speed must be positive and finite".to_string(),
        ));
    }
    if !thrust_available_n.is_finite() || thrust_available_n < 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "thrust must be finite and non-negative".to_string(),
        ));
    }
    let atmosphere = atmosphere(altitude_meters)?;
    let weight = aircraft.weight_n();
    let dynamic_pressure = atmosphere.dynamic_pressure(speed_m_s);
    let denominator = dynamic_pressure * aircraft.wing_area_m2;
    if denominator <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "dynamic pressure is not usable".to_string(),
        ));
    }
    let lift_coefficient = weight / denominator;
    let induced = lift_coefficient * lift_coefficient
        / (core::f64::consts::PI * aircraft.oswald_efficiency * aircraft.aspect_ratio);
    let drag_coefficient = aircraft.zero_lift_drag + induced;
    let drag_n = dynamic_pressure * aircraft.wing_area_m2 * drag_coefficient;
    let power_required_w = drag_n * speed_m_s;
    let climb_rate_m_s = (thrust_available_n - drag_n) * speed_m_s / weight;
    let fuel_flow_kg_s = thrust_available_n * aircraft.tsfc_kg_per_n_s;
    let lift_to_drag = lift_coefficient / drag_coefficient;
    Ok(LevelFlight {
        atmosphere,
        speed_m_s,
        mach: atmosphere.mach(speed_m_s),
        lift_coefficient,
        drag_coefficient,
        drag_n,
        thrust_required_n: drag_n,
        thrust_available_n,
        power_required_w,
        climb_rate_m_s,
        fuel_flow_kg_s,
        lift_to_drag,
    })
}

/// The Breguet cruise range and endurance of a jet aircraft.
///
/// 喷气式航空器的布雷盖巡航航程与航时。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CruiseRange {
    /// Still-air range in metres.
    ///
    /// 静风航程，单位为米。
    pub range_meters: f64,
    /// Endurance in seconds.
    ///
    /// 航时，单位为秒。
    pub endurance_seconds: f64,
    /// Lift-to-drag ratio used by the calculation.
    ///
    /// 计算所用的升阻比。
    pub lift_to_drag: f64,
}

/// Computes the Breguet range and endurance.
///
/// The aircraft must have a positive fuel consumption and the end mass must
/// be smaller than the start mass. The result follows from
/// `R = (V / (g c_t)) (L/D) ln(m0 / m1)` and `E = R / V`.
///
/// 计算布雷盖航程与航时。
///
/// 航空器须有正油耗，且末质量小于起始质量。结果由
/// `R = (V / (g c_t)) (L/D) ln(m0 / m1)` 与 `E = R / V` 得出。
pub fn breguet_cruise(
    aircraft: &Aircraft,
    speed_m_s: f64,
    lift_to_drag: f64,
    start_mass_kg: f64,
    end_mass_kg: f64,
) -> Result<CruiseRange> {
    if !aircraft.is_valid() {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "aircraft parameters must be positive and finite".to_string(),
        ));
    }
    if aircraft.tsfc_kg_per_n_s <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "thrust-specific fuel consumption must be positive".to_string(),
        ));
    }
    if !speed_m_s.is_finite() || speed_m_s <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "speed must be positive and finite".to_string(),
        ));
    }
    if !lift_to_drag.is_finite() || lift_to_drag <= 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "lift-to-drag ratio must be positive and finite".to_string(),
        ));
    }
    if !start_mass_kg.is_finite()
        || !end_mass_kg.is_finite()
        || end_mass_kg <= 0.0
        || start_mass_kg <= end_mass_kg
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "masses must be finite with 0 < end mass < start mass".to_string(),
        ));
    }
    let endurance_seconds = lift_to_drag / (STANDARD_GRAVITY * aircraft.tsfc_kg_per_n_s)
        * (start_mass_kg / end_mass_kg).ln();
    Ok(CruiseRange {
        range_meters: speed_m_s * endurance_seconds,
        endurance_seconds,
        lift_to_drag,
    })
}
