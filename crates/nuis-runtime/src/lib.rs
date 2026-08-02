//! Nuis AOT-side execution scaffolding.
//!
//! This crate exists only as local execution-side support for validated AOT
//! artifacts. It does not define execution topology, and it is not the
//! external `yalivia` project.

pub mod bridge;
pub mod error;
pub mod executor;
pub mod host_yir;
pub mod lifecycle_bootstrap;
pub mod loader;
pub mod registry;
pub mod session;

pub use bridge::{BridgeExecutor, PreparedDomainExecution};
pub use error::RuntimeError;
pub use executor::{
    ExecutionClockGate, ExecutionClockValidation, ExecutionContract, ExecutionPhaseAction,
    ExecutionPhaseBinding, ExecutionPhaseContext, ExecutionPhaseOutcome, ExecutionPlan,
    ExecutionProfile, ExecutionResourceBinding, ExecutionResourceKind, ExecutionStateSnapshot,
    ExecutionTrace, ExecutionTraceEvent, Executor,
};
pub use host_yir::{
    execute_host_yir_module, execute_host_yir_source, HostYirExecutionSummary, HostYirValueSummary,
};
pub use lifecycle_bootstrap::{
    plan_lifecycle_bootstrap, AppliedRelocationFacts, LifecycleBootstrapFacts,
    LifecycleBootstrapPlan, LifecycleBootstrapStage, MappedSectionFacts,
    RuntimeServiceBindingFacts, CLOCK_ROOT_BINDING_ID, CLOCK_ROOT_CONTRACT, GLM_ROOT_BINDING_ID,
    GLM_ROOT_CONTRACT, LIFECYCLE_BOOTSTRAP_ENTRY_KIND, LIFECYCLE_BOOTSTRAP_PLAN_IDENTITY_CONTRACT,
    LIFECYCLE_BOOTSTRAP_PLAN_PROTOCOL,
};
pub use loader::RuntimeLoader;
pub use registry::{AdapterRegistry, DomainAdapter};
pub use session::{
    ClockProtocolRuntimeSummary, HostConsumableDomainUnit, HostConsumableSummary, LoadedExecutable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRole {
    Verify,
    Bind,
    Execute,
    Reverify,
}
