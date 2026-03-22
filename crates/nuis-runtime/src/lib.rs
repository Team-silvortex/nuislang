//! Nuis AOT-side execution scaffolding.
//!
//! This crate exists only as local execution-side support for validated AOT
//! artifacts. It does not define execution topology, and it is not the
//! external `yalivia` project.

pub mod executor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRole {
    Verify,
    Bind,
    Execute,
    Reverify,
}
