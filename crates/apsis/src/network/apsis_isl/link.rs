//! The link geometry type and the state of a link.
//!
//! 链路几何类型以及链路的状态。

use crate::apsis_math::Vec3;

/// The geometry of a link between two satellites.
///
/// The range is the straight line distance, which a link budget uses for the
/// free space path loss. The two pointing vectors are the directions each
/// terminal must aim, expressed in the frame the positions were given in.
///
/// 两颗卫星之间链路的几何。
///
/// 距离是直线距离，链路预算以之计算自由空间路径损耗。两个指向矢量是各
/// 终端须瞄准的方向，表达在位置所给的坐标系中。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkGeometry {
    /// The distance between the two satellites, in kilometers.
    ///
    /// 两颗卫星之间的距离，单位为千米。
    pub range_km: f64,
    /// The unit vector from the first satellite toward the second.
    ///
    /// 自第一颗卫星指向第二颗的单位矢量。
    pub direction_first_to_second: Vec3,
    /// The unit vector from the second satellite toward the first.
    ///
    /// 自第二颗卫星指向第一颗的单位矢量。
    pub direction_second_to_first: Vec3,
    /// The closest the line between the two passes to the centre of the
    /// body, in kilometers.
    ///
    /// This is the quantity that decides whether the link is blocked. It is
    /// reported because a link that clears the limb by a small margin is
    /// fragile: atmospheric refraction and the oblateness of the body both
    /// eat into the margin.
    ///
    /// 两点连线与天体中心的最短距离，单位为千米。
    ///
    /// 它决定该链路是否被遮挡。之所以报告，是因为仅以很小余量擦过临边的
    /// 链路是脆弱的：大气折射与天体扁率都会侵蚀该余量。
    pub centre_distance_km: f64,
}

impl LinkGeometry {
    /// Returns the grazing margin of the link, in kilometers.
    ///
    /// This is how far the line between the two satellites clears the
    /// surface, measured at the closest point. A negative value means the
    /// link is blocked.
    ///
    /// 返回该链路的擦地余量，单位为千米。
    ///
    /// 它是两点连线在最接近处越过地表的量度。负值表示该链路被遮挡。
    #[must_use]
    pub fn surface_clearance(&self, body_radius: f64) -> f64 {
        self.centre_distance_km - body_radius
    }

    /// Returns `true` when the link clears the body.
    ///
    /// 当该链路越过天体时返回 `true`。
    #[must_use]
    pub fn is_clear(&self, body_radius: f64) -> bool {
        self.centre_distance_km >= body_radius
    }
}

/// The state of a link at one instant.
///
/// A link is either carrying traffic or not. The reasons a link can be
/// unavailable are kept separate, because routing across a constellation has
/// to distinguish a link that is geometrically blocked from one that is
/// merely not established yet.
///
/// 某一瞬时链路的状态。
///
/// 链路或承载业务或否。链路不可用的各种原因分开保留，因为在星座上路由
/// 须区分被几何遮挡的链路与尚未建立的链路。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkState {
    /// The link is geometrically clear and may be used.
    ///
    /// 该链路几何上通畅，可以使用。
    Available,
    /// The Earth blocks the line between the two satellites.
    ///
    /// 地球遮挡两颗卫星之间的连线。
    Blocked,
    /// The Sun is within the field of view of one of the terminals, so an
    /// optical link cannot be used.
    ///
    /// 太阳位于某一终端的视场内，故该光链路不可用。
    SunInField,
    /// The link is clear but is not established, for example because it has
    /// not yet been acquired or its capacity is exhausted.
    ///
    /// 该链路通畅但尚未建立，例如尚未捕获或其容量已耗尽。
    NotEstablished,
}

impl LinkState {
    /// Returns `true` when the link may carry traffic.
    ///
    /// 当该链路可承载业务时返回 `true`。
    #[must_use]
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Available)
    }

    /// Returns a short name for the state.
    ///
    /// 返回该状态的简称。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Blocked => "blocked",
            Self::SunInField => "sun in field",
            Self::NotEstablished => "not established",
        }
    }
}
