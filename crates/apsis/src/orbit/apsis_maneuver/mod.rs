//! Impulsive and finite orbital maneuver models.
//!
//! A maneuver changes an orbit by applying thrust. The two models here are
//! the two ends of the spectrum.
//!
//! An impulsive maneuver, [`Impulse`], applies a velocity change
//! instantaneously, so the position is unchanged and only the velocity
//! jumps. This is the model for a chemical burn, whose duration is short
//! compared with the orbital period, and it is the model every textbook
//! delta-v budget is written in. It is exact in the limit of zero burn time
//! and the source of the closed form results for transfers between two
//! orbits.
//!
//! A finite maneuver, [`FiniteBurn`], applies a constant acceleration over a
//! duration, so the position drifts during the burn and the achieved
//! velocity change is less than the thrust would give in free space when
//! the thrust has to fight gravity. It is the model for an electric
//! thruster, whose burn may last a large fraction of an orbit, and for any
//! burn where the short duration approximation is not good enough.
//!
//! The two agree in the limit: a finite burn of acceleration `a` over a
//! duration `t` delivers the impulse `a t`, so a [`FiniteBurn`] with a
//! vanishing duration converges on the matching [`Impulse`]. The tests check
//! that limit, which is what ties the two models together.
//!
//! The impulsive model also provides the classic two impulse transfer
//! between two coplanar circular orbits, [`hohmann_transfer`], which is the
//! reference every low thrust result is compared against. It gives the two
//! burns and the total delta-v for a transfer between two circular orbits of
//! given radii, together with the transfer time, the half period of the
//! transfer ellipse.
//!
//! Conventions match the rest of the orbit layer: lengths are in kilometers,
//! velocities in kilometers per second, and angles in radians.
//!
//! ```
//! use apsis::apsis_maneuver::{Impulse, hohmann_transfer};
//! use apsis::apsis_kepler::GravityParameter;
//! use apsis::apsis_math::Vec3;
//!
//! // An impulsive burn straight along the velocity.
//! let impulse = Impulse::new(Vec3::new(0.0, 1.0, 0.0)).expect("the change is usable");
//! assert!((impulse.magnitude() - 1.0).abs() < 1e-12);
//!
//! // The two burns that raise a circular orbit from 7000 to 8000 km.
//! let transfer = hohmann_transfer(7_000.0, 8_000.0, GravityParameter::WGS84)
//!     .expect("the radii are usable");
//! assert!(transfer.first_burn > 0.0);
//! assert!(transfer.total_delta_v() > 0.0);
//! ```
//!
//! 脉冲式与有限式轨道机动模型。
//!
//! 机动通过施加推力改变轨道。此处的两个模型是谱系的两端。
//!
//! 脉冲机动 [`Impulse`] 瞬时施加速度变化，故位置不变、只有速度跃变。这是
//! 化学推进点火所用的模型，其持续时间相对轨道周期很短，也是每本教科书
//! delta-v 预算所写的模型。它在点火时间为零的极限下精确，也是两轨道之间
//! 转移各闭式结果的来源。
//!
//! 有限机动 [`FiniteBurn`] 在一段时长上施加恒定加速度，故位置在点火期间
//! 漂移，且当推力须对抗引力时，所达到的速度变化小于推力在自由空间中会给出
//! 的。这是电推进器所用的模型，其点火可能持续轨道的很大一部分，也适用于
//! 任何短时近似不够好的点火。
//!
//! 二者在极限下一致：加速度为 `a`、时长为 `t` 的有限点火给出冲量 `a t`，
//! 故时长趋于零的 [`FiniteBurn`] 收敛到相应的 [`Impulse`]。测试检查该极限，
//! 正是它把两个模型联系起来。
//!
//! 脉冲模型还提供两个共面圆轨道之间经典的双脉冲转移
//! [`hohmann_transfer`]，它是每个低推力结果所比较的参照。它给出给定半径的
//! 两个圆轨道之间转移的两次点火与总 delta-v，以及转移时间，即转移椭圆的
//! 半个周期。
//!
//! 约定与 orbit 层其余部分一致：长度以千米计，速度以千米每秒计，角度以
//! 弧度计。

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod burn;
mod error;
mod hohmann;
mod impulse;

pub use burn::{BurnOutcome, FiniteBurn};
pub use error::{ManeuverError, ManeuverErrorKind};
pub use hohmann::{HohmannTransfer, hohmann_transfer};
pub use impulse::Impulse;

pub use crate::apsis_kepler::{GravityParameter, KeplerianElements};
pub use crate::apsis_math::Vec3;
