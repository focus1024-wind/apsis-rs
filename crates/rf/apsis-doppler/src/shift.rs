//! The Doppler shift and the range rate that causes it.
//!
//! 多普勒频移以及造成它的距离变化率。

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
///
/// 返回两终端之间的距离变化率，单位为米每秒。
///
/// 距离变化率是相对速度沿两终端连线的分量。正值表示间距在增大。
///
/// 第一个参数是远端终端相对本地终端的位置，故其模即间距。第二个是远端
/// 终端相对本地终端的速度。两者均为 SI 单位。
///
/// 间距为零时返回 `None`，因为那里没有定义视线；或任一参数非有限时
/// 返回 `None`。
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
/// <div>
/// $$
/// \Delta f = -f_0 \frac{v_r}{c}
/// $$
/// </div>
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
///
/// 返回某频率对应某距离变化率的经典多普勒频移，单位为赫兹。
///
/// 关系为
///
/// <div>
/// $$
/// \Delta f = -f_0 \frac{v_r}{c}
/// $$
/// </div>
///
/// 其中 `v_r` 是距离变化率，远离时为正。远离的终端其频移为负，故接收
/// 频率低于发射频率。
///
/// 该频移的幅度相对载波很小：12 GHz 下 7 km/s 的距离变化率约给出
/// 280 kHz，即十万分之二。故调用方必须把频率当作双精度处理，且移位后的
/// 频率是单独返回，而非把一个小频移加到一个大数上。
///
/// 频率非正且非有限，或距离变化率非有限时返回 `None`。
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
///
/// 返回某距离变化率的相对论多普勒因子。
///
/// 该因子为 `sqrt((1 - beta) / (1 + beta))`，其中 `beta = v_r / c`。把
/// 发射频率乘以它便给出对 `beta` 各阶都正确的接收频率，故它不是对经典
/// 频移的小修正，而是经典频移所近似的量。
///
/// 其展开为 `1 - beta + beta^2 / 2 - ...`。前两项是经典结果，它在本身为
/// 二阶的因子之内即 `1 - beta`；其余项是两种处理之差。对轨道速度的
/// 卫星，`beta` 约为 `2.3e-5`，二阶项约为载波的 `2.7e-10`，故两者相符
/// 到十的十次方分之几。
///
/// 距离变化率非有限，或其幅度达到光速（此时该式无实值）时返回 `None`。
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
///
/// 施加于载波的多普勒频移。
///
/// 该类型带有载波频率与距离变化率，故发射频率、接收频率以及频移本身都
/// 可得，而不必对所用符号约定存疑。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DopplerShift {
    /// The transmitted frequency, in hertz.
    ///
    /// 发射频率，单位为赫兹。
    carrier: f64,
    /// The range rate, in meters per second, positive when receding.
    ///
    /// 距离变化率，单位为米每秒，远离时为正。
    range_rate: f64,
}

impl DopplerShift {
    /// Creates a shift from a carrier frequency and a range rate.
    ///
    /// Returns `None` when the frequency is not positive and finite, or when
    /// the range rate is not finite.
    ///
    /// 由载波频率与距离变化率创建频移。
    ///
    /// 频率非正且非有限，或距离变化率非有限时返回 `None`。
    pub fn from_range_rate(carrier_frequency: f64, range_rate: f64) -> Option<Self> {
        classical_shift(carrier_frequency, range_rate)?;
        Some(Self {
            carrier: carrier_frequency,
            range_rate,
        })
    }

    /// Returns the transmitted frequency, in hertz.
    ///
    /// 返回发射频率，单位为赫兹。
    pub const fn carrier_frequency(self) -> f64 {
        self.carrier
    }

    /// Returns the range rate, in meters per second, positive when receding.
    ///
    /// 返回距离变化率，单位为米每秒，远离时为正。
    pub const fn range_rate(self) -> f64 {
        self.range_rate
    }

    /// Returns the shift, in hertz.
    ///
    /// The value is negative when the terminals are receding.
    ///
    /// 返回频移，单位为赫兹。
    ///
    /// 两终端远离时该值为负。
    pub fn shift(self) -> f64 {
        classical_shift(self.carrier, self.range_rate).unwrap_or(0.0)
    }

    /// Returns the received frequency, in hertz.
    ///
    /// 返回接收频率，单位为赫兹。
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
    ///
    /// 返回含相对论钟改正的频移，单位为赫兹。
    ///
    /// 接收频率是载波乘以相对论因子，故频移为 `f_0 (factor - 1)`。
    ///
    /// 这不是经典频移加一个小修正：该因子已含一阶项，其展开为
    /// `1 - beta + beta^2 / 2 - ...`，其首项恰是经典频移。经典值是对它的
    /// 一阶近似，两者只差二阶及更高阶项，量级为 `beta^2`，对卫星而言是
    /// 载波的十的十次方分之几。
    ///
    /// 先求因子、后乘载波，因为它与一之差极小，若在改正之前把频移加到
    /// 载波上会损失精度。
    ///
    /// 距离变化率达到光速时返回 `None`。
    pub fn shift_including_relativity(self) -> Option<f64> {
        let factor = relativistic_doppler_factor(self.range_rate)?;
        Some(self.carrier * (factor - 1.0))
    }

    /// Returns the shift in parts per million of the carrier.
    ///
    /// This is a convenient way to compare the shift of links on different
    /// carriers.
    ///
    /// 返回频移，以载波的百万分率计。
    ///
    /// 这是比较位于不同载波上的链路频移的便利方式。
    pub fn shift_in_ppm(self) -> f64 {
        self.shift() / self.carrier * 1.0e6
    }

    /// Returns the rate of change of the shift, in hertz per second.
    ///
    /// This is what a Doppler tracker must follow, and it follows from the
    /// range acceleration.
    ///
    /// Returns `None` when the acceleration is not finite.
    ///
    /// 返回频移的变化率，单位为赫兹每秒。
    ///
    /// 这是多普勒跟踪器必须跟随的量，它由距离加速度得出。
    ///
    /// 加速度非有限时返回 `None`。
    pub fn shift_rate(self, range_acceleration: f64) -> Option<f64> {
        if !range_acceleration.is_finite() {
            return None;
        }
        Some(-self.carrier * range_acceleration / SPEED_OF_LIGHT)
    }

    /// Returns `true` when the shift is a redshift, meaning the terminals
    /// are receding and the received frequency is lower.
    ///
    /// 当该频移是红移（即两终端远离、接收频率更低）时返回 `true`。
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
///
/// 返回视线与速度之间的夹角，单位为弧度。
///
/// 它是二者所在平面内通常仰角的余角，当已知距离变化率且已知来自轨道的
/// 速度时有用：该角由二者之比得出。
///
/// 结果落在 `[0, pi]` 内。
///
/// 速度为零长或任一参数非有限时返回 `None`。
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
///
/// 返回某载波与某速度下可能的最大频移幅度，单位为赫兹。
///
/// 这是运动完全沿视线时的频移，即接收机必须覆盖的捕获范围宽度。它是在
/// 接近与远离两个极端之间的总跨度的一半。
///
/// 结果以正幅度返回。载波或速度非正且非有限时返回 `None`。
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
///
/// 返回接收机为覆盖直到给定速度的每个距离变化率所必须接受的总频率跨度。
///
/// 接近与远离给出符号相反的频移，故该跨度是最大频移的两倍。
///
/// 载波或速度不可用时返回 `None`。
pub fn acquisition_span(carrier_frequency: f64, speed: f64) -> Option<f64> {
    maximum_shift(carrier_frequency, speed).map(|shift| 2.0 * shift)
}
