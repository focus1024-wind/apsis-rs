//! Writes the two-body conversions this workspace computes.
//!
//! The output is CSV on standard output, with a header row. It is consumed
//! by `validation/python/orbit/check_kepler.py`, which recomputes the same
//! quantities from the closed-form expressions and compares. The Python
//! side is written from the geometry rather than from this code, so a
//! transcription error shows up as a disagreement.
//!
//! Two tables are written. The first converts Keplerian elements to a
//! Cartesian state over a spread of orbits, which is the path a planning
//! caller uses. The second converts the other way and records the recovered
//! elements, which exercises the vector algebra and the conic section
//! relations rather than a single formula.
//!
//! 写出本工作区计算的二体换算。
//!
//! 输出是标准输出上的 CSV，带一行表头。它由
//! `validation/python/orbit/check_kepler.py` 消费，后者从闭式表达式重算相同的量
//! 并比较。Python 侧是依据几何而非依据本代码写成的，故转录错误会表现为不一致。
//!
//! 写出两张表。第一张在一系列轨道上把开普勒根数转成直角坐标状态，这是规划调用方
//! 所用的路径。第二张往另一方向转并记录所恢复的根数，它检验的是矢量代数与圆锥
//! 曲线关系，而非单一公式。
//!
//! Usage:
//!
//! ```text
//! cargo run --features orbit --example dump_kepler \
//!     > validation/data/expected/rust_kepler.csv
//! ```

use apsis::apsis_kepler::{GravityParameter, KeplerianElements};

/// The gravitational parameter used throughout, WGS 84.
///
/// 全程所用的引力参数，WGS 84。
const MU: GravityParameter = GravityParameter::WGS84;

/// One orbit to convert, chosen to span the shapes the model must handle.
///
/// 一个要转换的轨道，其选取跨越该模型必须处理的各形状。
struct Case {
    /// A name for the case, so a failure in the report is readable.
    ///
    /// 用例的名称，使报告中的失败可读。
    name: &'static str,
    /// Semi-major axis, in kilometers.
    ///
    /// 半长轴，单位为千米。
    semi_major_axis: f64,
    /// Eccentricity, dimensionless.
    ///
    /// 偏心率，无量纲。
    eccentricity: f64,
    /// Inclination, in degrees.
    ///
    /// 倾角，单位为度。
    inclination_deg: f64,
    /// Right ascension of the ascending node, in degrees.
    ///
    /// 升交点赤经，单位为度。
    right_ascension_deg: f64,
    /// Argument of perigee, in degrees.
    ///
    /// 近地点幅角，单位为度。
    argument_of_perigee_deg: f64,
    /// True anomaly, in degrees.
    ///
    /// 真近点角，单位为度。
    true_anomaly_deg: f64,
}

impl Case {
    /// Returns the case as elements in the units the crate uses.
    ///
    /// 以本 crate 所用的单位把该用例返回为根数。
    fn elements(&self) -> KeplerianElements {
        KeplerianElements {
            semi_major_axis: self.semi_major_axis,
            eccentricity: self.eccentricity,
            inclination: self.inclination_deg.to_radians(),
            right_ascension: self.right_ascension_deg.to_radians(),
            argument_of_perigee: self.argument_of_perigee_deg.to_radians(),
            true_anomaly: self.true_anomaly_deg.to_radians(),
        }
    }
}

/// The orbits converted.
///
/// The set spans a circular orbit, two mildly eccentric ones, a highly
/// eccentric one that exercises the near-parabolic branch of the anomaly
/// iteration, a retrograde one, and one inclined close to the polar limit.
///
/// 被转换的各轨道。
///
/// 该集合跨越一个圆轨道、两个轻度偏心者、一个检验近点角迭代近抛物线分支的
/// 高度偏心者、一个逆行轨道，以及一个倾角接近极地极限的轨道。
const CASES: &[Case] = &[
    Case {
        name: "circular_equatorial",
        semi_major_axis: 7_000.0,
        eccentricity: 0.0,
        inclination_deg: 0.0,
        right_ascension_deg: 0.0,
        argument_of_perigee_deg: 0.0,
        true_anomaly_deg: 0.0,
    },
    Case {
        name: "low_eccentricity",
        semi_major_axis: 7_000.0,
        eccentricity: 0.01,
        inclination_deg: 51.6,
        right_ascension_deg: 68.8,
        argument_of_perigee_deg: 22.9,
        true_anomaly_deg: 40.1,
    },
    Case {
        name: "moderate_eccentricity",
        semi_major_axis: 26_600.0,
        eccentricity: 0.74,
        inclination_deg: 5.7,
        right_ascension_deg: 114.6,
        argument_of_perigee_deg: 286.5,
        true_anomaly_deg: 63.0,
    },
    Case {
        name: "highly_eccentric",
        semi_major_axis: 26_000.0,
        eccentricity: 0.97,
        inclination_deg: 28.5,
        right_ascension_deg: 300.0,
        argument_of_perigee_deg: 10.0,
        true_anomaly_deg: 170.0,
    },
    Case {
        name: "retrograde",
        semi_major_axis: 8_000.0,
        eccentricity: 0.2,
        inclination_deg: 120.0,
        right_ascension_deg: 45.0,
        argument_of_perigee_deg: 200.0,
        true_anomaly_deg: 300.0,
    },
    Case {
        name: "near_polar",
        semi_major_axis: 7_300.0,
        eccentricity: 0.005,
        inclination_deg: 88.0,
        right_ascension_deg: 15.0,
        argument_of_perigee_deg: 95.0,
        true_anomaly_deg: 200.0,
    },
];

/// The geostationary radius, used for a circular orbit at that altitude.
///
/// 地球静止轨道半径，用于该高度处的圆轨道。
const GEO_RADIUS: f64 = 42_164.0;

fn main() {
    // The first table: elements to a Cartesian state.
    // 第一张表：根数到直角坐标状态。
    println!(
        "case,semi_major_axis_km,eccentricity,inclination_rad,right_ascension_rad,\
         argument_of_perigee_rad,true_anomaly_rad,x_km,y_km,z_km,vx_kms,vy_kms,vz_kms,\
         period_seconds,energy_kms2"
    );
    for case in CASES {
        let elements = case.elements();
        let state = elements
            .to_state(MU)
            .expect("the case describes an elliptical orbit");
        // The specific energy is reported as well, because it is the
        // quantity the Python side uses to check the speed independently of
        // the direction.
        // 比能量也被报告，因为它是 Python 侧用来独立于方向核对速率的量。
        let radius = state.position.norm();
        let energy = state.velocity.norm_squared() / 2.0 - MU.value() / radius;
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            case.name,
            case.semi_major_axis,
            case.eccentricity,
            case.inclination_deg.to_radians(),
            case.right_ascension_deg.to_radians(),
            case.argument_of_perigee_deg.to_radians(),
            case.true_anomaly_deg.to_radians(),
            state.position.x,
            state.position.y,
            state.position.z,
            state.velocity.x,
            state.velocity.y,
            state.velocity.z,
            elements.period(MU),
            energy
        );
    }

    // A circular orbit at the geostationary radius, whose speed has a closed
    // form, is included in the same table under its own name.
    // 地球静止半径处的圆轨道，其速率有闭式，故以它自己的名称纳入同一张表。
    let geo = KeplerianElements {
        semi_major_axis: GEO_RADIUS,
        eccentricity: 0.0,
        inclination: 0.0,
        right_ascension: 0.0,
        argument_of_perigee: 0.0,
        true_anomaly: 1.0,
    };
    let state = geo.to_state(MU).expect("geostationary circle is valid");
    let radius = state.position.norm();
    let energy = state.velocity.norm_squared() / 2.0 - MU.value() / radius;
    println!(
        "geostationary_circular,{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        GEO_RADIUS,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        state.position.x,
        state.position.y,
        state.position.z,
        state.velocity.x,
        state.velocity.y,
        state.velocity.z,
        geo.period(MU),
        energy
    );

    // The second table: a state back to elements, over a grid of true
    // anomalies on one orbit. This exercises the vector algebra rather than
    // the perifocal formulae the first table uses.
    // 第二张表：在一个轨道的真近点角网格上从状态转回根数。它检验矢量代数，而非
    // 第一张表所用的近焦点公式。
    println!();
    println!(
        "true_anomaly_deg,recovered_semi_major_axis_km,recovered_eccentricity,\
              recovered_inclination_rad,recovered_right_ascension_rad,\
              recovered_argument_of_perigee_rad,recovered_true_anomaly_rad,state_radius_km"
    );
    let base = KeplerianElements {
        semi_major_axis: 7_000.0,
        eccentricity: 0.3,
        inclination: 0.7,
        right_ascension: 1.2,
        argument_of_perigee: 0.4,
        true_anomaly: 0.0,
    };
    for step in 0..36 {
        let true_anomaly = f64::from(step) * 10.0_f64.to_radians();
        let elements = base.with_true_anomaly(true_anomaly);
        let state = elements
            .to_state(MU)
            .expect("the orbit is valid at every angle");
        let recovered = KeplerianElements::from_state(&state, MU).expect("the state is elliptical");
        println!(
            "{},{},{},{},{},{},{},{}",
            true_anomaly,
            recovered.semi_major_axis,
            recovered.eccentricity,
            recovered.inclination,
            recovered.right_ascension,
            recovered.argument_of_perigee,
            recovered.true_anomaly,
            state.position.norm()
        );
    }
}
