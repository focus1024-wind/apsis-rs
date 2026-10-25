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


/// Result of a steady level-turn calculation.
///
/// 稳定水平转弯计算结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurnPerformance {
    /// Load factor, in g.
    pub load_factor: f64,
    /// Bank angle, in degrees.
    pub bank_angle_deg: f64,
    /// Turn rate, in radians per second.
    pub turn_rate_rad_s: f64,
    /// Turn radius, in meters.
    pub turn_radius_m: f64,
    /// Time for a full 360-degree turn, in seconds.
    pub turn_time_s: f64,
}

/// Computes steady level-turn performance at a true airspeed.
///
/// The load factor must be at least one; the bank angle follows from
/// `n = 1 / cos(bank)`. Returns an error for non-finite or out-of-range input.
///
/// 计算某真空速下的稳定水平转弯性能。
///
/// 过载须至少为 1；坡度由 `n = 1 / cos(bank)` 得出。输入非有限或越界时返回错误。
pub fn turn_performance(speed_m_s: f64, load_factor: f64) -> Result<TurnPerformance> {
    if !speed_m_s.is_finite()
        || speed_m_s <= 0.0
        || !load_factor.is_finite()
        || load_factor < 1.0
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "speed must be positive and load factor must be at least one",
        ));
    }
    let lateral = (load_factor * load_factor - 1.0).sqrt();
    let turn_rate_rad_s = STANDARD_GRAVITY * lateral / speed_m_s;
    let turn_radius_m = speed_m_s * speed_m_s / (STANDARD_GRAVITY * lateral);
    Ok(TurnPerformance {
        load_factor,
        bank_angle_deg: (1.0 / load_factor).acos().to_degrees(),
        turn_rate_rad_s,
        turn_radius_m,
        turn_time_s: core::f64::consts::TAU / turn_rate_rad_s,
    })
}


/// A complete climb-cruise-descent mission profile.
///
/// 完整爬升-巡航-下降任务剖面。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MissionProfile {
    /// Climb segment.
    pub climb: ClimbSegment,
    /// Descent segment.
    pub descent: ClimbSegment,
    /// Cruise time, in seconds.
    pub cruise_time_s: f64,
    /// Cruise fuel, in kilograms.
    pub cruise_fuel_kg: f64,
    /// Total time, in seconds.
    pub total_time_s: f64,
    /// Total fuel, in kilograms.
    pub total_fuel_kg: f64,
    /// Total horizontal distance, in meters.
    pub total_distance_m: f64,
}

/// Computes a climb-cruise-descent mission profile.
///
/// 计算爬升-巡航-下降任务剖面。
#[allow(clippy::too_many_arguments)]
pub fn mission_profile(
    aircraft: &Aircraft,
    start_altitude_m: f64,
    cruise_altitude_m: f64,
    end_altitude_m: f64,
    cruise_distance_m: f64,
    speed_m_s: f64,
    climb_thrust_n: f64,
    descent_thrust_n: f64,
    step_m: f64,
) -> Result<MissionProfile> {
    if !cruise_distance_m.is_finite() || cruise_distance_m < 0.0 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "cruise distance must be non-negative and finite",
        ));
    }
    if start_altitude_m > cruise_altitude_m + 1e-9 || end_altitude_m > cruise_altitude_m + 1e-9 {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "start and end altitudes must not exceed cruise altitude",
        ));
    }
    let climb = climb_segment(aircraft, start_altitude_m, cruise_altitude_m, speed_m_s, climb_thrust_n, step_m)?;
    let descent = climb_segment(aircraft, cruise_altitude_m, end_altitude_m, speed_m_s, descent_thrust_n, step_m)?;
    let cruise_flight = level_flight(aircraft, cruise_altitude_m, speed_m_s, climb_thrust_n)?;
    let cruise_time_s = cruise_distance_m / speed_m_s;
    let cruise_fuel_kg = cruise_flight.thrust_required_n * aircraft.tsfc_kg_per_n_s * cruise_time_s;
    Ok(MissionProfile {
        climb,
        descent,
        cruise_time_s,
        cruise_fuel_kg,
        total_time_s: climb.time_s + cruise_time_s + descent.time_s,
        total_fuel_kg: climb.fuel_kg + cruise_fuel_kg + descent.fuel_kg,
        total_distance_m: climb.horizontal_distance_m + cruise_distance_m + descent.horizontal_distance_m,
    })
}

/// Result of a stepped climb or descent segment.
///
/// 分段爬升或下降结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClimbSegment {
    /// Start altitude, in meters.
    pub start_altitude_m: f64,
    /// End altitude, in meters.
    pub end_altitude_m: f64,
    /// Elapsed time, in seconds.
    pub time_s: f64,
    /// Horizontal distance, in meters.
    pub horizontal_distance_m: f64,
    /// Fuel burned, in kilograms.
    pub fuel_kg: f64,
    /// Average vertical rate, in meters per second.
    pub average_climb_rate_m_s: f64,
}

/// Integrates a climb or descent in constant-altitude steps.
///
/// Each step evaluates [`level_flight`] at the current altitude and uses its
/// climb rate and fuel flow. The segment is rejected when the requested
/// direction cannot be sustained by the available thrust, or when the inputs
/// are not usable.
///
/// 以等高步长积分爬升或下降。
///
/// 每步在当前高度调用 [`level_flight`]，使用其爬升率与燃油流量。若可用推力
/// 无法维持所需方向，或输入不可用，则返回错误。
pub fn climb_segment(
    aircraft: &Aircraft,
    start_altitude_m: f64,
    end_altitude_m: f64,
    speed_m_s: f64,
    thrust_available_n: f64,
    step_m: f64,
) -> Result<ClimbSegment> {
    if !start_altitude_m.is_finite()
        || !end_altitude_m.is_finite()
        || !step_m.is_finite()
        || step_m <= 0.0
    {
        return Err(Error::with_message(
            ErrorKind::InvalidInput,
            "altitudes and step must be finite, and step must be positive",
        ));
    }
    // Validate through a level-flight call even for a zero-length segment.
    level_flight(aircraft, start_altitude_m, speed_m_s, thrust_available_n)?;
    if (end_altitude_m - start_altitude_m).abs() < 1e-9 {
        return Ok(ClimbSegment {
            start_altitude_m,
            end_altitude_m,
            time_s: 0.0,
            horizontal_distance_m: 0.0,
            fuel_kg: 0.0,
            average_climb_rate_m_s: 0.0,
        });
    }
    let direction = (end_altitude_m - start_altitude_m).signum();
    let mut altitude = start_altitude_m;
    let mut time_s = 0.0;
    let mut distance_m = 0.0;
    let mut fuel_kg = 0.0;
    while (end_altitude_m - altitude).abs() > 1e-9 {
        let step = ((end_altitude_m - altitude).abs()).min(step_m) * direction;
        let flight = level_flight(aircraft, altitude, speed_m_s, thrust_available_n)?;
        if direction > 0.0 && flight.climb_rate_m_s <= 0.0 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "available thrust cannot sustain the climb",
            ));
        }
        if direction < 0.0 && flight.climb_rate_m_s >= 0.0 {
            return Err(Error::with_message(
                ErrorKind::InvalidInput,
                "available thrust cannot sustain the descent",
            ));
        }
        let dt = step / flight.climb_rate_m_s;
        time_s += dt;
        distance_m += speed_m_s * dt;
        fuel_kg += flight.fuel_flow_kg_s * dt;
        altitude += step;
    }
    Ok(ClimbSegment {
        start_altitude_m,
        end_altitude_m,
        time_s,
        horizontal_distance_m: distance_m,
        fuel_kg,
        average_climb_rate_m_s: (end_altitude_m - start_altitude_m) / time_s,
    })
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
