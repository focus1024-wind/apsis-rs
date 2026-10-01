//! The link budget and the figures derived from it.
//!
//! 链路预算以及由它导出的各数字。

use apsis_core::constant::BOLTZMANN;
use apsis_noise::NoiseTemperature;
use apsis_rf::{Decibel, Gain, Power, free_space_path_loss, wavelength};

/// The geometric part of a link: how far apart the terminals are and at what
/// frequency they operate.
///
/// 链路的几何部分：两端相距多远，以及在什么频率上工作。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkGeometry {
    /// The distance between the terminals, in meters.
    ///
    /// 两端之间的距离，单位为米。
    pub distance: f64,
    /// The carrier frequency, in hertz.
    ///
    /// 载波频率，单位为赫兹。
    pub frequency: f64,
}

impl LinkGeometry {
    /// Returns the free space path loss for this geometry.
    ///
    /// Returns `None` when the distance or frequency is not usable.
    ///
    /// 返回该几何的自由空间路径损耗。
    ///
    /// 距离或频率不可用时返回 `None`。
    pub fn path_loss(self) -> Option<Decibel> {
        free_space_path_loss(self.distance, self.frequency)
    }

    /// Returns `true` when the geometry is usable.
    ///
    /// 当该几何可用时返回 `true`。
    pub fn is_valid(self) -> bool {
        self.distance.is_finite()
            && self.distance > 0.0
            && self.frequency.is_finite()
            && self.frequency > 0.0
    }
}

/// A complete link budget for one direction of a radio link.
///
/// The losses are given as positive decibel values that are subtracted,
/// which is the convention in link budgets: a three decibel loss is entered
/// as three decibels, not as minus three.
///
/// 无线电链路一个方向的完整链路预算。
///
/// 各损耗以被减去的正分贝值给出，这是链路预算中的约定：三分贝的损耗记
/// 为三，而非负三。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkBudget {
    /// The power delivered to the transmit antenna, in watts.
    ///
    /// 送到发射天线的功率，单位为瓦特。
    pub transmit_power: Power,
    /// The transmit antenna gain on the relevant path.
    ///
    /// 相关路径上的发射天线增益。
    pub transmit_gain: Gain,
    /// Every loss between the transmitter and the transmit antenna, as a
    /// positive decibel value.
    ///
    /// 发射机与发射天线之间的每一项损耗，为正分贝值。
    pub transmit_loss: Decibel,
    /// The receive antenna gain on the relevant path.
    ///
    /// 相关路径上的接收天线增益。
    pub receive_gain: Gain,
    /// Every loss between the receive antenna and the receiver input, as a
    /// positive decibel value.
    ///
    /// 接收天线与接收机输入之间的每一项损耗，为正分贝值。
    pub receive_loss: Decibel,
    /// The system noise temperature referred to the receiver input.
    ///
    /// 折算到接收机输入端的系统噪声温度。
    pub system_temperature: NoiseTemperature,
    /// The receiver noise bandwidth, in hertz.
    ///
    /// 接收机噪声带宽，单位为赫兹。
    pub bandwidth: f64,
    /// The distance and frequency of the link.
    ///
    /// 链路的距离与频率。
    pub geometry: LinkGeometry,
}

impl LinkBudget {
    /// Returns the equivalent isotropically radiated power, in watts.
    ///
    /// This is the transmit power reduced by the transmit losses and
    /// increased by the transmit antenna gain. It is the power that an
    /// isotropic radiator would have to radiate to produce the same flux
    /// density in the direction of the receiver.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 返回等效全向辐射功率，单位为瓦特。
    ///
    /// 这是发射功率减去发射损耗、加上发射天线增益。它是一个各向同性辐射体为
    /// 在接收机方向上产生相同通量密度所必须辐射的功率。
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn eirp(&self) -> Option<Power> {
        if !self.transmit_loss.is_finite() || !self.geometry.is_valid() {
            return None;
        }
        let gain = self.transmit_gain.as_decibel()?;
        let net = gain - self.transmit_loss;
        self.transmit_power.scaled(net)
    }

    /// Returns the carrier power at the receiver input, in watts.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 返回接收机输入端的载波功率，单位为瓦特。
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn received_power(&self) -> Option<Power> {
        if !self.receive_loss.is_finite() {
            return None;
        }
        let eirp = self.eirp()?;
        let receive_gain = self.receive_gain.as_decibel()?;
        let loss = self.geometry.path_loss()?;
        eirp.scaled(receive_gain - loss - self.receive_loss)
    }

    /// Returns the noise power density at the receiver input, in watts per
    /// hertz.
    ///
    /// This is `k T`, the noise temperature converted to a power per unit
    /// bandwidth. It does not depend on the receiver bandwidth, which is
    /// what makes the carrier to noise density ratio a property of the link
    /// rather than of the receiver.
    ///
    /// Returns `None` when the system temperature is not usable.
    ///
    /// 返回接收机输入端的噪声功率密度，单位为瓦特每赫兹。
    ///
    /// 这是 `k T`，即把噪声温度换算为单位带宽的功率。它不依赖于接收机
    /// 带宽，这正是载噪密度比成为链路属性而非接收机属性的原因。
    ///
    /// 系统温度不可用时返回 `None`。
    pub fn noise_density(&self) -> Option<f64> {
        if !self.system_temperature.is_valid() {
            return None;
        }
        Some(BOLTZMANN * self.system_temperature.kelvin())
    }

    /// Returns the carrier to noise density ratio.
    ///
    /// This is the carrier power divided by the noise power in one hertz.
    /// It is the form used to compare a link against the energy per bit
    /// requirement of a modulation, because the data rate is accounted for
    /// separately.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 返回载噪密度比。
    ///
    /// 这是载波功率除以一赫兹内的噪声功率。它是把链路与某调制的每比特能量要求
    /// 相比较所用的形式，因为数据速率被分开计入。
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn carrier_to_noise_density(&self) -> Option<Decibel> {
        let carrier = self.received_power()?;
        let density = self.noise_density()?;
        if density <= 0.0 {
            return None;
        }
        // The ratio is a power ratio, and the numerator and denominator
        // carry different bandwidths, so the ratio is formed directly.
        // 该比值是功率比，且分子与分母带有不同的带宽，故直接构成该比。
        let ratio = carrier.watts() / density;
        Decibel::from_ratio(ratio)
    }

    /// Returns the carrier to noise ratio in the receiver bandwidth.
    ///
    /// This is the carrier to noise density ratio reduced by the bandwidth.
    /// Unlike the density ratio it depends on the receiver, which is why the
    /// same link can show different carrier to noise ratios in receivers of
    /// different bandwidth.
    ///
    /// Returns `None` when any input is not usable.
    ///
    /// 返回接收机带宽内的载噪比。
    ///
    /// 这是载噪密度比减去带宽的贡献。与密度比不同，它依赖于接收机，这正是同一
    /// 链路在不同带宽的接收机中可显示不同载噪比的原因。
    ///
    /// 任一输入不可用时返回 `None`。
    pub fn carrier_to_noise(&self) -> Option<Decibel> {
        if !self.bandwidth.is_finite() || self.bandwidth <= 0.0 {
            return None;
        }
        let density = self.carrier_to_noise_density()?;
        Some(density - Decibel::from_ratio(self.bandwidth)?)
    }

    /// Returns the energy per bit to noise density ratio for a data rate.
    ///
    /// This is the carrier to noise density ratio reduced by the data rate,
    /// because energy per bit is carrier power divided by bit rate.
    ///
    /// Returns `None` when the data rate is not positive, or when the
    /// carrier to noise density ratio is not usable.
    ///
    /// 返回某数据速率下每比特能量与噪声密度之比。
    ///
    /// 它是载噪密度比减去数据速率，因为每比特能量是载波功率除以比特
    /// 速率。
    ///
    /// 数据速率非正，或载噪密度比不可用时返回 `None`。
    pub fn energy_per_bit_over_noise(&self, data_rate: f64) -> Option<Decibel> {
        if !data_rate.is_finite() || data_rate <= 0.0 {
            return None;
        }
        let density = self.carrier_to_noise_density()?;
        Some(density - Decibel::from_ratio(data_rate)?)
    }

    /// Returns the rise in system noise caused by the receive antenna
    /// viewing a noise source of the given temperature.
    ///
    /// The antenna noise is added to the system temperature in power, which
    /// in temperature is a sum. This is the form used to account for the
    /// Earth, the Sun, or a rain cell in the beam.
    ///
    /// Returns `None` when the added temperature is not usable.
    ///
    /// 返回接收天线看到给定温度的噪声源所引起的系统噪声上升。
    ///
    /// 天线噪声按功率加到系统温度上，在温度上即求和。这是用于计入波束中
    /// 的地球、太阳或雨胞的形式。
    ///
    /// 所加温度不可用时返回 `None`。
    pub fn with_added_noise(&self, added: NoiseTemperature) -> Option<Self> {
        if !added.is_valid() {
            return None;
        }
        Some(Self {
            system_temperature: self.system_temperature + added,
            ..*self
        })
    }

    /// Returns the wavelength of the carrier, in meters.
    ///
    /// Returns `None` when the frequency is not usable.
    ///
    /// 返回载波的波长，单位为米。
    ///
    /// 频率不可用时返回 `None`。
    pub fn carrier_wavelength(&self) -> Option<f64> {
        wavelength(self.geometry.frequency)
    }

    /// Returns `true` when every input describes a usable link.
    ///
    /// 当每个输入都描述一条可用链路时返回 `true`。
    pub fn is_valid(&self) -> bool {
        self.transmit_power.is_valid()
            && self.transmit_gain.dbi().is_some()
            && self.transmit_loss.is_finite()
            && self.receive_gain.dbi().is_some()
            && self.receive_loss.is_finite()
            && self.system_temperature.is_valid()
            && self.bandwidth.is_finite()
            && self.bandwidth > 0.0
            && self.geometry.is_valid()
    }
}

/// The result of comparing a link against a requirement.
///
/// 把链路与某项要求比较所得的结果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkMargin {
    /// The ratio the link achieves, in decibels.
    ///
    /// 该链路所达到的比，单位为分贝。
    pub achieved: Decibel,
    /// The ratio the link requires, in decibels.
    ///
    /// 该链路所要求的比，单位为分贝。
    pub required: Decibel,
}

impl LinkMargin {
    /// Returns the margin, in decibels.
    ///
    /// A positive margin means the link closes with room to spare; a
    /// negative margin means it does not.
    ///
    /// 返回余量，单位为分贝。
    ///
    /// 正余量意味着链路闭合并有余，负余量意味着不能闭合。
    pub fn margin(self) -> Decibel {
        self.achieved - self.required
    }

    /// Returns `true` when the margin is not negative.
    ///
    /// 当余量非负时返回 `true`。
    pub fn closes(self) -> bool {
        self.margin().db() >= 0.0
    }
}

/// Returns the margin between an achieved ratio and a required one.
///
/// This is the comparison that decides whether a link closes.
///
/// 返回所达到的比与所要求的比之间的余量。
///
/// 这一比较决定链路是否闭合。
pub fn link_margin(achieved: Decibel, required: Decibel) -> LinkMargin {
    LinkMargin { achieved, required }
}

/// Returns the carrier to noise density ratio for a budget.
///
/// This is a free function form of
/// [`LinkBudget::carrier_to_noise_density`].
///
/// Returns `None` when the budget is not usable.
///
/// 返回某预算的载噪密度比。
///
/// 这是 [`LinkBudget::carrier_to_noise_density`] 的自由函数形式。
///
/// 预算不可用时返回 `None`。
pub fn carrier_to_noise_density(budget: &LinkBudget) -> Option<Decibel> {
    budget.carrier_to_noise_density()
}

/// Returns the gain to system temperature ratio of a receive chain, in
/// reciprocal kelvin.
///
/// This figure of merit is written `G / T` and is the standard way to
/// compare receive stations. It combines the receive antenna gain with the
/// system noise temperature, so a station with a bigger dish and a quieter
/// receiver scores higher.
///
/// Expressed in decibels, `G / T` is the receive gain in dBi minus ten times
/// the logarithm of the temperature, which is the form usually quoted.
///
/// Returns `None` when the gain or temperature is not usable.
///
/// 返回接收链路的增益与系统温度之比，单位为每开尔文。
///
/// 该品质因数写作 `G / T`，是比较接收站的标准方式。它把接收天线增益与
/// 系统噪声温度结合起来，故天线更大、接收机更安静的站得分更高。
///
/// 以分贝表示时，`G / T` 是接收增益（以 dBi 计）减去温度的十倍对数，
/// 这是通常引用的形式。
///
/// 增益或温度不可用时返回 `None`。
pub fn gain_to_temperature(gain: Gain, temperature: NoiseTemperature) -> Option<f64> {
    if !temperature.is_valid() || temperature.kelvin() <= 0.0 {
        return None;
    }
    let ratio = gain.ratio() / temperature.kelvin();
    if ratio.is_finite() && ratio > 0.0 {
        Some(ratio)
    } else {
        None
    }
}
