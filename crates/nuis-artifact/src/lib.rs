mod artifact;
mod bridge_registry;
mod build_manifest;
mod clock_protocol;
mod compiler_candidate_execution;
mod compiler_candidate_production;
mod compiler_component_build;
mod compiler_component_diff;
mod compiler_component_reproducibility;
mod compiler_diagnostic_report;
mod compiler_stage_handoff;
mod compiler_stage_semantic_differential;
mod compiler_stage_transformation;
mod compiler_structural_projection;
mod compiler_structural_projection_page;
mod compiler_token_decoder;
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
pub use compiler_component_reproducibility::{
    build_compiler_component_reproducibility, build_compiler_component_reproducibility_from_paths,
    parse_compiler_component_reproducibility, parse_compiler_component_reproducibility_from_source,
    read_compiler_component_reproducibility, render_compiler_component_reproducibility,
    CompilerComponentReproducibility, CompilerComponentReproducibilityRootInput,
    CompilerComponentReproducibilityRun, CompilerComponentReproducibilityRunInput,
    COMPILER_COMPONENT_CLEAN_BUILD_CONTRACT, COMPILER_COMPONENT_REPRODUCIBILITY_AUTHORITY,
    COMPILER_COMPONENT_REPRODUCIBILITY_FILE, COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL,
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
pub use compiler_stage_semantic_differential::{
    build_compiler_stage_semantic_differential, parse_compiler_stage_semantic_differential,
    parse_compiler_stage_semantic_differential_from_source,
    read_compiler_stage_semantic_differential, render_compiler_stage_semantic_differential,
    verify_compiler_stage_semantic_differential, CompilerStageSemanticComparison,
    CompilerStageSemanticDifferential, CompilerStageSemanticDifferentialInput,
    COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_AUTHORITY, COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE,
    COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PRODUCER_CONTRACT,
    COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PROTOCOL, COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_VERDICT,
    COMPILER_STAGE_SEMANTIC_EQUIVALENCE_CONTRACT,
};
pub use compiler_stage_transformation::{
    build_compiler_stage_transformations, compiler_projection_checkpoint_kind_tag,
    compiler_stage_structural_checkpoint_words, compiler_stage_transformation_payload_file,
    encode_compiler_stage_transformation_payload,
    materialize_compiler_stage_transformation_payloads, parse_compiler_stage_transformations,
    parse_compiler_stage_transformations_from_source, read_compiler_stage_transformations,
    render_compiler_stage_transformations, verify_compiler_stage_transformations,
    CompilerStageTransformationRecord, CompilerStageTransformationRecordInput,
    CompilerStageTransformations, CompilerStageTransformationsInput,
    COMPILER_STAGE_CHECKPOINT_PAGE_COUNT, COMPILER_STAGE_CHECKPOINT_WORD_COUNT,
    COMPILER_STAGE_STRUCTURAL_CHECKPOINT_CONTRACT, COMPILER_STAGE_TRANSFORMATION_AUTHORITY,
    COMPILER_STAGE_TRANSFORMATION_FILE, COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
    COMPILER_STAGE_TRANSFORMATION_PRODUCER_CONTRACT, COMPILER_STAGE_TRANSFORMATION_PROTOCOL,
};
pub use compiler_structural_projection::{
    parse_compiler_structural_projection, render_compiler_structural_projection,
    verify_compiler_projection_identity, CompilerProjectionKind, CompilerProjectionRecord,
    CompilerProjectionRecordKind, CompilerStructuralProjection, COMPILER_AST_PROJECTION_ENCODING,
    COMPILER_NIR_PROJECTION_ENCODING, COMPILER_STRUCTURAL_PROJECTION_CONTRACT,
};
pub use compiler_structural_projection_page::{
    compiler_projection_first_page_identity, compiler_projection_resume_page_identity,
    compiler_projection_two_page_identity, CompilerProjectionPageAdvance,
    CompilerProjectionPageCursor, CompilerProjectionPageIdentity,
    CompilerProjectionTwoPageIdentity, COMPILER_PROJECTION_CURSOR_CONTRACT,
    COMPILER_PROJECTION_CURSOR_HASH_SEED, COMPILER_PROJECTION_CURSOR_LANES,
    COMPILER_PROJECTION_PAGE_BODY_HASH_SEED, COMPILER_PROJECTION_PAGE_BYTES,
    COMPILER_PROJECTION_PAGE_CONTRACT, COMPILER_PROJECTION_PAGE_HASH_MODULUS,
    COMPILER_PROJECTION_PAGE_HASH_SEED, COMPILER_PROJECTION_PAGE_IDENTITY_RADIX,
};
pub use compiler_token_decoder::{
    compiler_token_first_page_identity, decode_compiler_token_stream, CompilerTokenDecodeSummary,
    CompilerTokenPageIdentity, COMPILER_TOKEN_DECODER_CONTRACT,
    COMPILER_TOKEN_DECODER_FOLD_MODULUS, COMPILER_TOKEN_DECODER_MAX_BYTES,
    COMPILER_TOKEN_DECODER_MAX_RECORDS, COMPILER_TOKEN_DECODER_SEMANTIC_SEED,
    COMPILER_TOKEN_PAGE_CANONICAL_BYTES, COMPILER_TOKEN_PAGE_IDENTITY_RADIX,
    COMPILER_TOKEN_PAGE_PAYLOAD_BYTES, COMPILER_TOKEN_PAGE_RECORDS, COMPILER_TOKEN_STREAM_PROTOCOL,
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
