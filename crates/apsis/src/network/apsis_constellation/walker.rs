//! The Walker constellation pattern and its instantiation.
//!
//! Walker 星座构型及其实例化。

use crate::apsis_constellation::error::{ConstellationError, ConstellationErrorKind};
use crate::apsis_kepler::{GravityParameter, KeplerianElements};
use crate::apsis_math::Vec3;
use core::f64::consts::PI;

/// A Walker delta pattern.
///
/// A Walker pattern places a number of satellites evenly around the Earth in
/// a number of planes that share an inclination and a semi-major axis. The
/// relative phasing of adjacent planes is set by the phase factor, so the
/// whole constellation is described by five numbers.
///
/// Walker 构型把若干卫星均匀分布在绕地球的若干轨道面上，各面共用同一倾角与
/// 同一半长轴。相邻轨道面之间的相对相位由相位因子决定，因此整个星座由五个数
/// 描述。
///
/// The pattern is written `i: t/p/f`, where `i` is the inclination, `t` the
/// total number of satellites, `p` the number of planes, and `f` the phase
/// factor. This structure stores the same quantities, with the inclination
/// in radians and the semi-major axis in kilometers.
///
/// 构型写作 `i: t/p/f`，其中 `i` 为倾角、`t` 为卫星总数、`p` 为轨道面数、
/// `f` 为相位因子。本结构体存储同样的量，倾角用弧度，半长轴用千米。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WalkerPattern {
    /// The total number of satellites.
    ///
    /// 卫星总数。
    pub total_satellites: usize,
    /// The number of orbital planes.
    ///
    /// 轨道面数。
    pub planes: usize,
    /// The phase factor, in the range `0..planes`.
    ///
    /// 相位因子，取值范围 `0..planes`。
    pub phase_factor: usize,
    /// The inclination of every plane, in radians.
    ///
    /// 各轨道面的倾角，单位为弧度。
    pub inclination: f64,
    /// The semi-major axis of every satellite, in kilometers.
    ///
    /// 各卫星的半长轴，单位为千米。
    pub semi_major_axis: f64,
}

impl WalkerPattern {
    /// Returns the number of satellites in each plane.
    ///
    /// Returns `None` when the satellite count is not a multiple of the
    /// plane count.
    ///
    /// 返回每个轨道面上的卫星数。
    ///
    /// 当卫星总数不是轨道面数的整数倍时返回 `None`。
    #[must_use]
    pub fn satellites_per_plane(self) -> Option<usize> {
        if self.planes == 0 || self.total_satellites % self.planes != 0 {
            return None;
        }
        Some(self.total_satellites / self.planes)
    }

    /// Returns the spacing between adjacent planes in right ascension, in
    /// radians.
    ///
    /// Returns `None` when the plane count is zero, which is the only case
    /// in which the spacing is undefined.
    ///
    /// The spacing does not depend on the pattern being consistent, so a
    /// caller can report the value it would use even for a pattern that
    /// cannot be instantiated.
    ///
    /// 返回相邻轨道面之间的赤经间隔，单位为弧度，即 `360 / planes` 度。
    ///
    /// 当轨道面数为零时返回 `None`，这是该间隔未定义的唯一情形。
    ///
    /// 该间隔不依赖构型是否自洽，因此即使构型无法实例化，调用方也能报告它本会
    /// 使用的值。
    #[must_use]
    pub fn plane_spacing(self) -> Option<f64> {
        (self.planes > 0).then(|| 2.0 * PI / self.planes as f64)
    }

    /// Returns the phase offset applied between adjacent planes, in radians.
    ///
    /// This is `f * 360 / t` degrees, where `f` is the phase factor and `t`
    /// the total satellite count.
    ///
    /// Returns `None` when the satellite count is zero, which is the only
    /// case in which the offset is undefined.
    ///
    /// 返回相邻轨道面之间施加的相位偏移，单位为弧度。
    ///
    /// 即 `f * 360 / t` 度，其中 `f` 为相位因子，`t` 为卫星总数。
    ///
    /// 当卫星总数为零时返回 `None`，这是该偏移未定义的唯一情形。
    #[must_use]
    pub fn phase_offset(self) -> Option<f64> {
        if self.total_satellites == 0 {
            return None;
        }
        Some(2.0 * PI * self.phase_factor as f64 / self.total_satellites as f64)
    }

    /// Returns `true` when the pattern can be instantiated.
    ///
    /// 当该构型可以被实例化时返回 `true`。
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.total_satellites > 0
            && self.planes > 0
            && self.total_satellites % self.planes == 0
            && self.phase_factor < self.planes
            && self.inclination.is_finite()
            && (0.0..=PI).contains(&self.inclination)
            && self.semi_major_axis.is_finite()
            && self.semi_major_axis > 0.0
    }

    /// Returns the Walker notation for this pattern, such as `53: 24/3/1`.
    ///
    /// The inclination is printed at full precision, so a value such as
    /// 53.25 degrees survives the round trip. The notation is an identifier
    /// rather than a rounded quotation, so truncating it would lose
    /// information for no gain.
    ///
    /// 返回该构型的 Walker 记法，例如 `53: 24/3/1`。
    ///
    /// 倾角按全精度打印，因此 53.25 度这样的值可以原样往返。该记法用作标识而
    /// 非取整后的引用值，截断它只会丢失信息而没有收益。
    #[must_use]
    pub fn notation(self) -> String {
        format!(
            "{}: {}/{}/{}",
            self.inclination.to_degrees(),
            self.total_satellites,
            self.planes,
            self.phase_factor
        )
    }

    /// Instantiates the pattern as a list of orbital elements.
    ///
    /// The mean anomaly of the reference satellite of the first plane is
    /// zero. Each further satellite in a plane is advanced by
    /// `360 / (t / p)` degrees, and each further plane is advanced by the
    /// phase offset.
    ///
    /// Returns an error describing why the pattern cannot be instantiated.
    ///
    /// Source: the phasing rule is the one defined in Walker, J. G.,
    /// *Circular Orbit Patterns Providing Continuous Whole Earth Coverage*,
    /// Royal Aircraft Establishment Technical Report 70211, 1970.
    ///
    /// 把该构型实例化为一组轨道根数。
    ///
    /// 第一个轨道面的参考卫星的平近点角为零。同一面内每增加一颗卫星，平近点角
    /// 前进 `360 / (t / p)` 度；每增加一个轨道面，前进一个相位偏移。
    ///
    /// 当该构型无法实例化时返回一个说明原因的错误。
    ///
    /// 出处：相位规则取自 Walker, J. G., *Circular Orbit Patterns Providing
    /// Continuous Whole Earth Coverage*, Royal Aircraft Establishment
    /// Technical Report 70211, 1970。
    ///
    /// # Panics
    ///
    /// Panics only if the pattern is inconsistent with the validation this
    /// call performs, which cannot happen.
    ///
    /// 仅当构型与本调用所做的校验不一致时 panic，这不会发生。
    pub fn instantiate(self) -> Result<Vec<KeplerianElements>, ConstellationError> {
        if self.total_satellites == 0 {
            return Err(ConstellationError::new(
                ConstellationErrorKind::NoSatellites,
            ));
        }
        if self.planes == 0 {
            return Err(ConstellationError::new(ConstellationErrorKind::NoPlanes));
        }
        let per_plane = self.satellites_per_plane().ok_or_else(|| {
            ConstellationError::new(ConstellationErrorKind::UnevenPlanes).with_detail(format!(
                "{} satellites cannot be spread over {} planes",
                self.total_satellites, self.planes
            ))
        })?;
        if !self.is_valid() {
            return Err(
                ConstellationError::new(ConstellationErrorKind::InvalidOrbit).with_detail(format!(
                    "inclination {} rad, semi-major axis {} km, phase factor {}",
                    self.inclination, self.semi_major_axis, self.phase_factor
                )),
            );
        }

        let plane_spacing = self.plane_spacing().expect("a positive plane count");
        let phase_offset = self.phase_offset().expect("a positive satellite count");
        let anomaly_spacing = 2.0 * PI / per_plane as f64;

        let mut satellites = Vec::with_capacity(self.total_satellites);
        for plane in 0..self.planes {
            // The right ascension of this plane.
            // 本轨道面的赤经。
            let right_ascension = (plane as f64 * plane_spacing).rem_euclid(2.0 * PI);
            // The phase offset of this plane relative to the first.
            // 本轨道面相对第一个轨道面的相位偏移。
            let plane_phase = plane as f64 * phase_offset;
            for satellite in 0..per_plane {
                let mean_anomaly =
                    (satellite as f64 * anomaly_spacing + plane_phase).rem_euclid(2.0 * PI);
                satellites.push(KeplerianElements {
                    semi_major_axis: self.semi_major_axis,
                    // The Walker pattern is circular.
                    // Walker 构型为圆轨道。
                    eccentricity: 0.0,
                    inclination: self.inclination,
                    right_ascension,
                    // The argument of perigee is undefined for a circular
                    // orbit, so it is set to zero and the position along the
                    // orbit is carried entirely by the mean anomaly.
                    // 圆轨道的近地点幅角未定义，故置零；轨道上的位置完全由
                    // 平近点角承载。
                    argument_of_perigee: 0.0,
                    true_anomaly: crate::apsis_kepler::true_from_mean(mean_anomaly, 0.0),
                });
            }
        }
        Ok(satellites)
    }
}

/// One satellite in an instantiated constellation.
///
/// 已实例化星座中的一颗卫星。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Satellite {
    /// The index of the satellite within the constellation.
    ///
    /// 该卫星在星座中的序号。
    pub index: usize,
    /// The index of the orbital plane it belongs to.
    ///
    /// 该卫星所属轨道面的序号。
    pub plane: usize,
    /// The index of the satellite within its plane.
    ///
    /// 该卫星在其所在轨道面内的序号。
    pub index_in_plane: usize,
    /// The orbital elements at the reference time.
    ///
    /// 历元时刻的轨道根数。
    pub elements: KeplerianElements,
}

/// One orbital plane sampled as Earth-fixed Cartesian points.
///
/// The points are produced from the first satellite of the plane, at the
/// constellation reference time. They are intended for visualization of the
/// plane, not for precise ground-track work.
///
/// 一个以地固笛卡尔点采样的轨道面。
///
/// 点由该轨道面的第一颗卫星在星座参考时刻产生，用于轨道面可视化，而不是
/// 精确的地面轨迹计算。
#[derive(Debug, Clone, PartialEq)]
pub struct PlaneOutline {
    /// The plane index.
    ///
    /// 轨道面序号。
    pub plane: usize,
    /// The sampled points, in meters in the Earth-fixed frame.
    ///
    /// 采样点，单位为米，位于地固系。
    pub points: Vec<Vec3>,
}

/// A constellation instantiated at a reference time.
///
/// The elements are the ones at the reference time passed to the
/// constructor. Advancing them in time is the job of the orbit layer, which
/// propagates each satellite independently.
///
/// 在参考时刻实例化的星座。
///
/// 根数即构造函数所给参考时刻的那一组。随时间推进由 orbit 层负责，该层对每颗
/// 卫星独立传播。
#[derive(Debug, Clone, PartialEq)]
pub struct Constellation {
    satellites: Vec<Satellite>,
    pattern: Option<WalkerPattern>,
    reference_mjd: f64,
}

impl Constellation {
    /// Instantiates a Walker pattern at a reference time.
    ///
    /// The reference time is a Modified Julian Date on the TAI scale. It is
    /// stored with the constellation and returned unchanged; the elements
    /// are the ones at that instant.
    ///
    /// Returns an error when the pattern cannot be instantiated.
    ///
    /// 在参考时刻实例化一个 Walker 构型。
    ///
    /// 参考时刻为 TAI 时间尺度上的修正儒略日（Modified Julian Date）。它随
    /// 星座一并保存并原样返回；根数即该时刻的根数。
    ///
    /// 当该构型无法实例化时返回一个错误。
    ///
    /// # Panics
    ///
    /// Panics only if the division check in `instantiate` disagrees with this
    /// call, which cannot happen.
    ///
    /// 仅当 `instantiate` 中的除法校验与本调用不一致时 panic，这不会发生。
    pub fn walker(pattern: WalkerPattern, reference_mjd: f64) -> Result<Self, ConstellationError> {
        if !reference_mjd.is_finite() {
            return Err(
                ConstellationError::new(ConstellationErrorKind::InvalidOrbit)
                    .with_detail("the reference time is not finite"),
            );
        }
        let elements = pattern.instantiate()?;
        let per_plane = pattern
            .satellites_per_plane()
            .expect("instantiation checked the division");

        let satellites = elements
            .into_iter()
            .enumerate()
            .map(|(index, elements)| Satellite {
                // The satellites are produced plane by plane, so the plane
                // index is the quotient and the position within it the
                // remainder.
                // 卫星按轨道面依次产生，故轨道面序号为商，面内序号为余数。
                index,
                plane: index / per_plane,
                index_in_plane: index % per_plane,
                elements,
            })
            .collect();

        Ok(Self {
            satellites,
            pattern: Some(pattern),
            reference_mjd,
        })
    }

    /// Builds a constellation from a list of satellites.
    ///
    /// This is the constructor for a constellation that does not follow the
    /// Walker pattern. The plane index of each satellite is derived from the
    /// right ascension of its elements, so satellites sharing a plane are
    /// grouped.
    ///
    /// Returns an error when the list is empty, when the reference time is
    /// not finite, or when any element set is not a valid orbit.
    ///
    /// 由一组卫星根数构建星座。
    ///
    /// 这是为不遵循 Walker 构型的星座提供的构造函数。每颗卫星的轨道面序号由其根数
    /// 的赤经推得，因此共用同一轨道面的卫星会被归为一组。
    ///
    /// 当列表为空、参考时刻非有限，或任一根数集不是有效轨道时返回一个错误。
    pub fn from_satellites(
        elements: &[KeplerianElements],
        reference_mjd: f64,
    ) -> Result<Self, ConstellationError> {
        if elements.is_empty() {
            return Err(ConstellationError::new(
                ConstellationErrorKind::NoSatellites,
            ));
        }
        if !reference_mjd.is_finite() {
            return Err(
                ConstellationError::new(ConstellationErrorKind::InvalidOrbit)
                    .with_detail("the reference time is not finite"),
            );
        }

        // Group by right ascension, assigning plane indices in order of
        // first appearance so that the numbering is stable.
        // 按赤经分组，轨道面序号按首次出现的顺序分配，使编号稳定。
        let mut planes: Vec<f64> = Vec::new();
        let mut satellites = Vec::with_capacity(elements.len());
        let mut counts: Vec<usize> = Vec::new();
        for (index, elements) in elements.iter().enumerate() {
            if !elements.is_valid() {
                return Err(
                    ConstellationError::new(ConstellationErrorKind::InvalidOrbit)
                        .with_detail(format!("satellite {index} has invalid elements")),
                );
            }
            let plane = if let Some(plane) = planes
                .iter()
                .position(|value| (value - elements.right_ascension).abs() < 1e-12)
            {
                plane
            } else {
                planes.push(elements.right_ascension);
                counts.push(0);
                planes.len() - 1
            };
            let index_in_plane = counts[plane];
            counts[plane] += 1;
            satellites.push(Satellite {
                index,
                plane,
                index_in_plane,
                elements: *elements,
            });
        }

        Ok(Self {
            satellites,
            pattern: None,
            reference_mjd,
        })
    }

    /// Samples each orbital plane as Earth-fixed Cartesian points.
    ///
    /// The first satellite of each plane supplies the elements. The sample
    /// starts at that satellite's mean anomaly and sweeps one full orbit, so
    /// `samples` intervals produce `samples + 1` points with the first and
    /// last point coincident. A value of zero returns no outlines.
    ///
    /// The inertial positions are rotated to the Earth-fixed frame with the
    /// Greenwich sidereal rotation at the constellation reference time. Polar
    /// motion is neglected. This is enough for drawing a plane ring, but it is
    /// not a substitute for a full frame chain when precise coordinates are
    /// needed.
    ///
    /// 把每个轨道面采样为地固笛卡尔点。
    ///
    /// 每个轨道面由其中第一颗卫星提供根数。采样从该卫星的平近点角开始，扫过
    /// 一整圈，因此 `samples` 个间隔产生 `samples + 1` 个点，首尾点重合。传入
    /// 零则不返回任何轮廓。
    ///
    /// 惯性位置按星座参考时刻的格林尼治恒星时转动到地固系，忽略极移。这足以
    /// 绘制轨道面环，但当需要精确坐标时不能替代完整的坐标链。
    #[must_use]
    pub fn plane_outlines(&self, samples: usize) -> Vec<PlaneOutline> {
        if samples == 0 {
            return Vec::new();
        }
        let julian_date = crate::apsis_time::julian_date_from_mjd(self.reference_mjd);
        let rotation = crate::apsis_frames::iau2000::teme_to_terrestrial_matrix(
            julian_date,
            0.0,
            crate::apsis_frames::iau2000::PolarMotion::NONE,
        );
        let mut outlines = Vec::with_capacity(self.plane_count());
        for plane in 0..self.plane_count() {
            let Some(satellite) = self
                .satellites
                .iter()
                .find(|satellite| satellite.plane == plane)
            else {
                continue;
            };
            let elements = satellite.elements;
            let start = elements.mean_anomaly();
            let mut points = Vec::with_capacity(samples + 1);
            for index in 0..=samples {
                let anomaly = start + core::f64::consts::TAU * index as f64 / samples as f64;
                let Some(state) = elements
                    .with_mean_anomaly(anomaly)
                    .to_state(GravityParameter::WGS84)
                else {
                    break;
                };
                points.push(rotation * state.position);
            }
            outlines.push(PlaneOutline { plane, points });
        }
        outlines
    }

    /// Returns the satellites in the constellation.
    ///
    /// 返回星座中的卫星。
    #[must_use]
    pub fn satellites(&self) -> &[Satellite] {
        &self.satellites
    }

    /// Returns the number of satellites.
    ///
    /// 返回卫星数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.satellites.len()
    }

    /// Returns `true` when the constellation is empty.
    ///
    /// A constellation built through either constructor is never empty, so
    /// this always returns `false`; it exists because a collection type
    /// should offer it alongside [`Constellation::len`].
    ///
    /// 经任一构造函数构建的星座都不会为空，故此函数恒返回 `false`；它存在是因为
    /// 集合类型应当与 [`Constellation::len`] 一并提供。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.satellites.is_empty()
    }

    /// Returns the number of orbital planes.
    ///
    /// 返回轨道面数。
    #[must_use]
    pub fn plane_count(&self) -> usize {
        self.satellites
            .iter()
            .map(|satellite| satellite.plane)
            .max()
            .map_or(0, |highest| highest + 1)
    }

    /// Returns the Walker pattern, when the constellation was built from
    /// one.
    ///
    /// 若星座由 Walker 构型构建则返回该构型，否则返回 `None`。
    #[must_use]
    pub fn pattern(&self) -> Option<WalkerPattern> {
        self.pattern
    }

    /// Returns the reference time as a Modified Julian Date on the TAI
    /// scale.
    ///
    /// 返回参考时刻，为 TAI 时间尺度上的修正儒略日（Modified Julian Date）。
    #[must_use]
    pub fn reference_mjd(&self) -> f64 {
        self.reference_mjd
    }

    /// Returns the satellites of one plane.
    ///
    /// Returns an empty slice when the plane index is out of range.
    ///
    /// 返回某个轨道面上的卫星。
    ///
    /// 当轨道面序号越界时返回空切片。
    #[must_use]
    pub fn plane(&self, plane: usize) -> Vec<Satellite> {
        self.satellites
            .iter()
            .filter(|satellite| satellite.plane == plane)
            .copied()
            .collect()
    }

    /// Returns the altitude of the constellation above the given radius, in
    /// kilometers.
    ///
    /// Every satellite of a Walker pattern shares a semi-major axis, so a
    /// single altitude describes the whole constellation. For a custom
    /// constellation this is the altitude of the first satellite.
    ///
    /// Returns `None` when the constellation is empty.
    ///
    /// Walker 构型中所有卫星共用同一半长轴，故单一高度即可描述整个星座；对自定义
    /// 星座，返回的是第一颗卫星的高度。
    ///
    /// 星座为空时返回 `None`。
    #[must_use]
    pub fn altitude_above(&self, body_radius_km: f64) -> Option<f64> {
        self.satellites
            .first()
            .map(|satellite| satellite.elements.semi_major_axis - body_radius_km)
    }

    /// Returns the count of satellites in each plane.
    ///
    /// The result has one entry per plane, in plane order.
    ///
    /// 返回的向量每个轨道面一项，按轨道面顺序排列。
    #[must_use]
    pub fn plane_populations(&self) -> Vec<usize> {
        let mut populations = vec![0; self.plane_count()];
        for satellite in &self.satellites {
            populations[satellite.plane] += 1;
        }
        populations
    }
}
