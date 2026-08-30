//! Minimum toolchain versions required to build a scaffolded Soroban project.
//!
//! Shared between `doctor` (which checks these on the host) and `scaffold`
//! (which pins the same versions into a generated `.devcontainer/`), so the
//! two can never drift apart.

/// Minimum Rust version able to target [`WASM_TARGET`].
pub const MIN_RUST: (u32, u32) = (1, 84);

/// Minimum `stellar-cli` version.
pub const MIN_STELLAR: (u32, u32) = (21, 0);

/// The wasm target Soroban contracts compile to.
pub const WASM_TARGET: &str = "wasm32v1-none";
