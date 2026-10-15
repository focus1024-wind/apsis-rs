//! The integration test suite of the crate.
//!
//! The suite mirrors the layer and module structure under `src/`. Every test
//! reaches the crate through a public path of the crate root, so a re-export
//! that goes missing fails here even when the module itself still compiles.
//!
//! 本 crate 的集成测试套件。
//!
//! 套件镜像 `src/` 下的层与模块结构。每个测试都经 crate 根的公开路径访问
//! 本 crate，故某个重导出缺失时，即使模块本身仍能编译，也会在此失败。

#[cfg(feature = "foundation")]
mod foundation;

#[cfg(feature = "physics")]
mod physics;

#[cfg(feature = "orbit")]
mod orbit;

#[cfg(feature = "rf")]
mod rf;

#[cfg(feature = "network")]
mod network;

#[cfg(feature = "ops")]
mod ops;

#[cfg(feature = "sim")]
mod sim;
