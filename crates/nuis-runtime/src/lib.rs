//! Nuis AOT-side execution scaffolding.
//!
//! This crate exists only as local execution-side support for validated AOT
//! artifacts. It does not define execution topology, and it is not the
//! external `yalivia` project.

pub mod bridge;
pub mod error;
pub mod executable_memory;
pub mod executor;
pub mod host_yir;
pub mod lifecycle_bootstrap;
pub mod lifecycle_execution;
pub mod loader;
pub mod registry;
pub mod session;

pub use bridge::{BridgeExecutor, PreparedDomainExecution};
pub use error::RuntimeError;
pub use executable_memory::{
    ExecutableEntryPreparation, ExecutableEntryRequest, ExecutableMemoryAdapter,
    NativeEntryInvocationResult, NativeHostExecutableMemoryAdapter,
    EXECUTABLE_MEMORY_ADAPTER_CONTRACT, NATIVE_ENTRY_INVOCATION_PROTOCOL,
    NUIS_LIFECYCLE_ENTRY_ABI_V1, NUIS_NATIVE_ENTRY_SECTION_KIND,
};
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
pub use lifecycle_execution::{
    prepare_lifecycle_bootstrap_execution, CompiledEntryTransferResult,
    LifecycleBootstrapExecutionPreparation, OwnedAppliedRelocationHandle, OwnedImageMapping,
    OwnedMappedSectionHandle, OwnedRuntimeServiceHandle, COMPILED_ENTRY_TRANSFER_PROTOCOL,
    LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT, LIFECYCLE_BOOTSTRAP_EXECUTION_PROTOCOL,
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
