mod artifact;
mod bridge_registry;
mod build_manifest;
mod clock_protocol;
mod compiler_candidate_execution;
mod compiler_candidate_production;
mod compiler_component_build;
mod compiler_component_diff;
mod compiler_diagnostic_report;
mod compiler_stage_handoff;
mod compiler_structural_projection;
mod domain_unit;
mod envelope;
mod error;
mod host_bridge_plan;
mod payload_blob;
pub mod protocol;
mod toml;

pub use artifact::{
    decode_nuis_compiled_artifact_binary, decode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_compiled_artifact_binary, encode_nuis_compiled_artifact_section_table,
    encode_nuis_compiled_artifact_section_table_binary, materialize_embedded_artifact_support,
    parse_nuis_compiled_artifact, parse_nuis_lowering_index_from_source,
    validate_compiled_artifact_section_table, write_nuis_compiled_artifact, NuisCompiledArtifact,
    NuisCompiledArtifactHostObject, NuisCompiledArtifactSection, NuisCompiledArtifactSectionTable,
    NuisLifecycleContract, NuisLoweringIndex, NuisLoweringIndexUnit,
};
pub use bridge_registry::{
    parse_bridge_registry, parse_bridge_registry_from_source, BridgeRegistry, BridgeRegistryEntry,
};
pub use build_manifest::{
    parse_build_manifest, parse_build_manifest_from_source, ArtifactHashEntry, BuildManifest,
};
pub use clock_protocol::{
    parse_clock_protocol, parse_clock_protocol_from_source, ClockDomain, ClockEdge, ClockProtocol,
};
pub use compiler_candidate_execution::{
    build_compiler_candidate_execution, parse_compiler_candidate_execution,
    parse_compiler_candidate_execution_from_source, read_compiler_candidate_execution,
    render_compiler_candidate_execution, CompilerCandidateExecution,
    CompilerCandidateExecutionInput, COMPILER_CANDIDATE_EXECUTION_AUTHORITY,
    COMPILER_CANDIDATE_EXECUTION_FILE, COMPILER_CANDIDATE_EXECUTION_PROTOCOL,
    COMPILER_CANDIDATE_EXECUTION_ROLE, COMPILER_CANDIDATE_RUNNER_CONTRACT,
};
pub use compiler_candidate_production::{
    build_compiler_candidate_production, compiler_candidate_bundle_fold,
    compiler_candidate_stage_fold, parse_compiler_candidate_production,
    parse_compiler_candidate_production_from_source, read_compiler_candidate_production,
    render_compiler_candidate_production, CompilerCandidateProduction,
    CompilerCandidateProductionInput, CompilerCandidateProductionRecord,
    COMPILER_CANDIDATE_ADAPTER_FILE, COMPILER_CANDIDATE_PRODUCER_CONTRACT,
    COMPILER_CANDIDATE_PRODUCTION_AUTHORITY, COMPILER_CANDIDATE_PRODUCTION_FILE,
    COMPILER_CANDIDATE_PRODUCTION_PROTOCOL,
};
pub use compiler_component_build::{
    build_compiler_component_build, parse_compiler_component_build,
    parse_compiler_component_build_from_source, promote_compiler_component_candidate,
    read_compiler_component_build, render_compiler_component_build,
    verify_compiler_component_build_image, CompilerComponentBuild, CompilerComponentBuildInput,
    CompilerComponentCandidatePromotionInput, CompilerComponentDependency,
    CompilerComponentDependencyInput, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_BUILD_PROTOCOL, COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT,
    COMPILER_COMPONENT_DRIVER_CONTRACT, COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
};
pub use compiler_component_diff::{
    build_compiler_component_differential, compare_compiler_component_paths,
    parse_compiler_component_differential, parse_compiler_component_differential_from_source,
    render_compiler_component_differential, CompilerComponentComparison,
    CompilerComponentDifferential, CompilerComponentEvidence, COMPILER_COMPONENT_DIFFERENTIAL_FILE,
    COMPILER_COMPONENT_DIFFERENTIAL_GATE_CONTRACT, COMPILER_COMPONENT_DIFFERENTIAL_PROTOCOL,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT,
};
pub use compiler_diagnostic_report::{
    build_compiler_diagnostic_report, parse_compiler_diagnostic_report,
    parse_compiler_diagnostic_report_from_source, read_compiler_diagnostic_report,
    render_compiler_diagnostic_report, CompilerDiagnosticInput, CompilerDiagnosticRecord,
    CompilerDiagnosticReport, CompilerDiagnosticReportInput,
    COMPILER_DIAGNOSTIC_NORMALIZATION_CONTRACT, COMPILER_DIAGNOSTIC_REPORT_FILE,
    COMPILER_DIAGNOSTIC_REPORT_PROTOCOL,
};
pub use compiler_stage_handoff::{
    build_compiler_stage_handoff, parse_compiler_stage_handoff_from_source,
    read_compiler_stage_handoff, render_compiler_stage_handoff, CompilerStageHandoff,
    CompilerStageHandoffRecord, CompilerStageKind, CompilerStagePayloadInput,
    VerifiedCompilerStagePayload, COMPILER_STAGE_HANDOFF_PROTOCOL,
    COMPILER_STAGE_PRODUCER_CONTRACT,
};
pub use compiler_structural_projection::{
    parse_compiler_structural_projection, render_compiler_structural_projection,
    verify_compiler_projection_identity, CompilerProjectionKind, CompilerProjectionRecord,
    CompilerProjectionRecordKind, CompilerStructuralProjection, COMPILER_AST_PROJECTION_ENCODING,
    COMPILER_NIR_PROJECTION_ENCODING, COMPILER_STRUCTURAL_PROJECTION_CONTRACT,
};
pub use domain_unit::{parse_domain_build_unit_blocks, BuildManifestDomainBuildUnit};
pub use envelope::{
    decode_nuis_executable_envelope_binary, encode_nuis_executable_envelope_binary,
    parse_nuis_executable_envelope, parse_nuis_executable_envelope_from_source,
    render_nuis_executable_envelope, write_nuis_executable_envelope, NuisExecutableEnvelope,
};
pub use error::ArtifactError;
pub use host_bridge_plan::{
    parse_host_bridge_plan_index, parse_host_bridge_plan_index_from_source, HostBridgePlanEntry,
    HostBridgePlanIndex,
};
pub use payload_blob::{
    decode_domain_payload_blob, encode_domain_payload_blob, DomainBuildUnitPayloadBlob,
    DomainBuildUnitPayloadBlobSection,
};
