//! Nuis AOT-side execution scaffolding.
//!
//! This crate exists only as local execution-side support for validated AOT
//! artifacts. It does not define execution topology, and it is not the
//! external `yalivia` project.

pub mod bridge;
pub mod error;
pub mod loader;
pub mod registry;
pub mod executor;
pub mod session;

pub use bridge::{BridgeExecutor, PreparedDomainExecution};
pub use error::RuntimeError;
pub use loader::RuntimeLoader;
pub use registry::{AdapterRegistry, DomainAdapter};
pub use session::LoadedExecutable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRole {
    Verify,
    Bind,
    Execute,
    Reverify,
}
