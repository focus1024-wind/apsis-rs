//! Free space propagation.
//!
//! 自由空间传播。

use crate::apsis_core::constant::SPEED_OF_LIGHT;
use crate::apsis_rf::decibel::Decibel;
use crate::apsis_rf::gain::Gain;
use core::f64::consts::PI;

/// Returns the wavelength of a frequency in vacuum, in meters.
///
/// The relation is `lambda = c / f`, with `c` exact by the definition of the
/// metre.
///
/// Returns `None` when the frequency is not positive and finite.
///
/// 返回某频率在真空中的波长，单位为米。
///
/// 关系为 `lambda = c / f`，其中 `c` 依米的定义是精确的。
///
/// 频率非正且非有限时返回 `None`。
#[must_use]
pub fn wavelength(frequency: f64) -> Option<f64> {
    if frequency > 0.0 && frequency.is_finite() {
        Some(SPEED_OF_LIGHT / frequency)
    } else {
        None
    }
}

/// Returns the free space path loss between two isotropic antennas, as a
/// ratio in decibels.
///
/// The ratio is positive, so it represents a loss: the received power is the
/// transmitted power divided by this ratio. The relation is
///
/// <div>
/// $$
/// L = \left(\frac{4 \pi d}{\lambda}\right)^2
/// $$
/// </div>
///
/// which is the spreading loss of a spherical wave over a distance `d`
/// combined with the effective aperture of an isotropic receive antenna.
///
/// The loss grows with frequency at a fixed distance, because the effective
/// aperture of the receive antenna falls as the square of the wavelength.
///
/// The distance must be positive. A zero distance is not a case this
/// expression covers: the ratio would be zero, which is infinite attenuation
/// in decibels, and the underlying far field assumption does not hold.
///
/// Returns `None` when the distance is not positive, or when either argument
/// is not finite.
///
/// 返回两个各向同性天线之间的自由空间路径损耗，以分贝比值表示。
///
/// 该比值为正，故表示一种损耗：接收功率是发射功率除以该比值。关系为
///
/// <div>
/// $$
/// L = \left(\frac{4 \pi d}{\lambda}\right)^2
/// $$
/// </div>
///
/// 它是球面波在距离 `d` 上的扩散损耗与各向同性接收天线有效孔径的
/// 结合。
///
/// 固定距离下损耗随频率增大，因为接收天线的有效孔径按波长的平方下降。
///
/// 距离必须为正。零距离不是本式所涵盖的情形：比值会为零，即以分贝计的
/// 无穷衰减，且其背后的远场假定不成立。
///
/// 距离非正，或任一参数非有限时返回 `None`。
#[must_use]
pub fn free_space_path_loss(distance: f64, frequency: f64) -> Option<Decibel> {
    if !distance.is_finite() || distance <= 0.0 {
        return None;
    }
    let lambda = wavelength(frequency)?;
    let ratio = 4.0 * PI * distance / lambda;
    Decibel::from_ratio(ratio * ratio)
}

/// Returns the received power ratio between two antennas in free space.
///
/// The received power is the transmitted power multiplied by the two antenna
/// gains and divided by the free space path loss, which in decibels is the
/// sum
///
/// <div>
/// $$
/// \frac{P_r}{P_t} = \frac{G_t G_r}{L}
/// $$
/// </div>
///
/// The result is the ratio in decibels, so a positive value means the
/// receive antenna collects more than was transmitted, which happens when
/// the two gains exceed the spreading loss.
///
/// Returns `None` when the distance or frequency is not usable.
///
/// 返回自由空间两个天线之间的接收功率比。
///
/// 接收功率是发射功率乘以两个天线增益、再除以自由空间路径损耗，以分贝
/// 表示即
///
/// <div>
/// $$
/// \frac{P_r}{P_t} = \frac{G_t G_r}{L}
/// $$
/// </div>
///
/// 结果是分贝比值，故正值意味着接收天线收集到的多于所发射的，这在两个
/// 增益超过扩散损耗时发生。
///
/// 距离或频率不可用时返回 `None`。
#[must_use]
pub fn free_space_received_ratio(
    distance: f64,
    frequency: f64,
    transmitter: Gain,
    receiver: Gain,
) -> Option<Decibel> {
    let loss = free_space_path_loss(distance, frequency)?;
    let transmitter = transmitter.as_decibel()?;
    let receiver = receiver.as_decibel()?;
    Some(transmitter + receiver - loss)
}
