//! Writes link budget inputs and the figures this workspace computes for
//! them.
//!
//! The output is CSV on standard output, with a header row. It is consumed
//! by `validation/python/link/check_link.py`, which recomputes the same
//! quantities from the link equation and compares.
//!
//! 写出链路预算输入以及本工作区为它们计算的各数字。
//!
//! 输出是标准输出上的 CSV，带一行表头。它由
//! `validation/python/link/check_link.py` 消费，后者从链路方程重算相同的量并
//! 比较。
//!
//! Usage:
//!
//! ```text
//! cargo run -p apsis-rs --features rf --example dump_link \
//!     > validation/data/expected/rust_link.csv
//! ```

use apsis_link::{LinkBudget, LinkGeometry};
use apsis_rf::{Decibel, Gain, Power};

/// One link scenario, named so that the fields stay readable at each use.
///
/// The unit of each field is in its name: powers and gains in decibels,
/// temperature in kelvin, and everything else in SI base units.
///
/// 一个链路场景，命名使各字段在每次使用时保持可读。
///
/// 每个字段的单位在其名称中：功率与增益以分贝计，温度以开尔文计，其余以 SI
/// 基本单位计。
struct Scenario {
    transmit_dbw: f64,
    transmit_dbi: f64,
    transmit_loss_db: f64,
    receive_dbi: f64,
    receive_loss_db: f64,
    temperature_k: f64,
    bandwidth_hz: f64,
    distance_m: f64,
    frequency_hz: f64,
}

/// The scenarios checked. They cover a Ku band downlink, an S band link, a
/// wideband Ka link, and an L band link with a warmer receiver.
///
/// 被检查的各场景。它们覆盖 Ku 频段下行、S 频段链路、宽带 Ka 链路，以及一个
/// 接收机更热的 L 频段链路。
const SCENARIOS: &[Scenario] = &[
    Scenario {
        transmit_dbw: 10.0,
        transmit_dbi: 30.0,
        transmit_loss_db: 1.0,
        receive_dbi: 35.0,
        receive_loss_db: 0.5,
        temperature_k: 150.0,
        bandwidth_hz: 36.0e6,
        distance_m: 38.0e6,
        frequency_hz: 12.0e9,
    },
    Scenario {
        transmit_dbw: 0.0,
        transmit_dbi: 20.0,
        transmit_loss_db: 2.0,
        receive_dbi: 25.0,
        receive_loss_db: 1.0,
        temperature_k: 290.0,
        bandwidth_hz: 1.0e6,
        distance_m: 1_000_000.0,
        frequency_hz: 2.2e9,
    },
    Scenario {
        transmit_dbw: 20.0,
        transmit_dbi: 45.0,
        transmit_loss_db: 0.5,
        receive_dbi: 50.0,
        receive_loss_db: 0.2,
        temperature_k: 80.0,
        bandwidth_hz: 500.0e6,
        distance_m: 400_000.0,
        frequency_hz: 20.0e9,
    },
    Scenario {
        transmit_dbw: 5.0,
        transmit_dbi: 15.0,
        transmit_loss_db: 3.0,
        receive_dbi: 10.0,
        receive_loss_db: 2.0,
        temperature_k: 500.0,
        bandwidth_hz: 10.0e6,
        distance_m: 36_000_000.0,
        frequency_hz: 1.5e9,
    },
];

fn main() {
    println!(
        "transmit_dbw,transmit_dbi,transmit_loss_db,receive_dbi,receive_loss_db,\
         temperature_k,bandwidth_hz,distance_m,frequency_hz,eirp_dbw,carrier_dbw,\
         cn0_db,cn_db,ebn0_db"
    );

    for scenario in SCENARIOS {
        let Scenario {
            transmit_dbw,
            transmit_dbi,
            transmit_loss_db: transmit_loss,
            receive_dbi,
            receive_loss_db: receive_loss,
            temperature_k: temperature,
            bandwidth_hz: bandwidth,
            distance_m: distance,
            frequency_hz: frequency,
        } = *scenario;

        let Some(transmit_power) = Power::from_dbw(transmit_dbw) else {
            continue;
        };
        let Some(transmit_gain) = Gain::from_dbi(transmit_dbi) else {
            continue;
        };
        let Some(receive_gain) = Gain::from_dbi(receive_dbi) else {
            continue;
        };

        let budget = LinkBudget {
            transmit_power,
            transmit_gain,
            transmit_loss: Decibel::from_db(transmit_loss),
            receive_gain,
            receive_loss: Decibel::from_db(receive_loss),
            system_temperature: apsis_link::NoiseTemperature::from_kelvin(temperature),
            bandwidth,
            geometry: LinkGeometry {
                distance,
                frequency,
            },
        };

        let eirp = budget.eirp().and_then(|power| power.dbw());
        let carrier = budget.received_power().and_then(|power| power.dbw());
        let cn0 = budget.carrier_to_noise_density().map(|value| value.db());
        let cn = budget.carrier_to_noise().map(|value| value.db());
        // The energy per bit ratio is reported at a fixed data rate equal to
        // the bandwidth, which makes it the carrier to noise ratio.
        //
        // 每比特能量比在等于带宽的固定数据速率下报告，这使它即为载噪比。
        let ebn0 = budget
            .energy_per_bit_over_noise(bandwidth)
            .map(|value| value.db());

        let field = |value: Option<f64>| match value {
            Some(value) => format!("{value:.12}"),
            None => "nan".to_string(),
        };
        println!(
            "{transmit_dbw},{transmit_dbi},{transmit_loss},{receive_dbi},{receive_loss},\
             {temperature},{bandwidth:.6},{distance:.6},{frequency:.6},\
             {},{},{},{},{}",
            field(eirp),
            field(carrier),
            field(cn0),
            field(cn),
            field(ebn0)
        );
    }
}
