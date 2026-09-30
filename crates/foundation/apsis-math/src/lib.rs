//! Dependency-free linear algebra for the ApsisRS workspace.
//!
//! The crate provides three types:
//!
//! - [`Vec3`], a three component column vector over `f64`
//! - [`Quat`], a unit quaternion used for rotations
//! - [`Mat3`], a three by three matrix stored in column-major order
//!
//! The types are plain data with no interior invariants beyond those
//! documented on each method, so their fields are public and a value can be
//! built by struct literal expression. Operations that are mathematically
//! undefined for some inputs, such as normalizing the zero vector, return
//! [`Option`] instead of panicking or producing a non-finite result.
//!
//! All rotations use the right-hand rule about the given axis, and all
//! angles are in radians. Angles are not wrapped on input; methods that
//! require a canonical range document that requirement.
//!
//! ```
//! use apsis_math::Vec3;
//!
//! let x = Vec3::new(1.0, 0.0, 0.0);
//! let y = Vec3::new(0.0, 1.0, 0.0);
//! let z = x.cross(y);
//! assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod mat3;
pub mod parallel;
mod quat;
mod vec3;

pub use mat3::Mat3;
pub use quat::Quat;
pub use vec3::Vec3;
