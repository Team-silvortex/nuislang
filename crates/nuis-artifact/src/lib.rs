mod artifact;
mod bridge_registry;
mod build_manifest;
mod clock_protocol;
mod compiler_candidate_compile_capability;
mod compiler_candidate_direct_compile;
mod compiler_candidate_execution;
mod compiler_candidate_fresh_source;
mod compiler_candidate_fresh_source_result;
mod compiler_candidate_frontend_result;
mod compiler_candidate_nsld_input;
mod compiler_candidate_nsld_materialization;
mod compiler_candidate_preselection;
mod compiler_candidate_production;
mod compiler_candidate_successor;
mod compiler_component_active_state;
mod compiler_component_attestation;
mod compiler_component_attestation_registry;
mod compiler_component_build;
mod compiler_component_compile_dispatch;
mod compiler_component_diff;
mod compiler_component_dispatch;
mod compiler_component_replacement;
mod compiler_component_replacement_registry;
mod compiler_component_representation_diff;
mod compiler_component_reproducibility;
mod compiler_component_transition;
mod compiler_diagnostic_report;
mod compiler_stage_handoff;
mod compiler_stage_handoff_v2;
mod compiler_stage_semantic_differential;
mod compiler_stage_transformation;
mod compiler_structural_projection;
mod compiler_structural_projection_page;
mod compiler_token_decoder;
mod compiler_token_pagination;
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
pub use compiler_candidate_compile_capability::{
    build_compiler_candidate_compile_capability, parse_compiler_candidate_compile_capability,
    parse_compiler_candidate_compile_capability_from_source,
    render_compiler_candidate_compile_capability, CompilerCandidateCompileCapability,
    CompilerCandidateCompileCapabilityInput, COMPILER_CANDIDATE_COMPILE_ADMISSION_CONTRACT,
    COMPILER_CANDIDATE_COMPILE_ARGUMENT_CONTRACT, COMPILER_CANDIDATE_COMPILE_CAPABILITY_AUTHORITY,
    COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE, COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL,
    COMPILER_CANDIDATE_COMPILE_CAPABILITY_VERDICT, COMPILER_CANDIDATE_COMPILE_COMMAND,
    COMPILER_CANDIDATE_COMPILE_DRIVER_CONTRACT, COMPILER_CANDIDATE_COMPILE_PROVIDER_CONTRACT,
    COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT, COMPILER_CANDIDATE_COMPILE_REQUEST_CONTRACT,
};
pub use compiler_candidate_direct_compile::{
    build_compiler_candidate_direct_compile_capability,
    parse_compiler_candidate_direct_compile_capability,
    parse_compiler_candidate_direct_compile_capability_from_source,
    render_compiler_candidate_direct_compile_capability,
    verify_compiler_candidate_direct_compile_capability, CompilerCandidateDirectCompileCapability,
    CompilerCandidateDirectCompileCapabilityInput,
    COMPILER_CANDIDATE_DIRECT_COMPILE_ARGUMENT_CONTRACT,
    COMPILER_CANDIDATE_DIRECT_COMPILE_AUTHORITY, COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE,
    COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL,
    COMPILER_CANDIDATE_DIRECT_COMPILE_DRIVER_CONTRACT,
    COMPILER_CANDIDATE_DIRECT_COMPILE_ENVIRONMENT_CONTRACT,
    COMPILER_CANDIDATE_DIRECT_COMPILE_INPUT_CONTRACT,
    COMPILER_CANDIDATE_DIRECT_COMPILE_NATIVE_CONTRACT,
    COMPILER_CANDIDATE_DIRECT_COMPILE_PROVIDER_CONTRACT,
    COMPILER_CANDIDATE_DIRECT_COMPILE_REQUEST_CONTRACT, COMPILER_CANDIDATE_DIRECT_COMPILE_VERDICT,
};
pub use compiler_candidate_execution::{
    build_compiler_candidate_execution, parse_compiler_candidate_execution,
    parse_compiler_candidate_execution_from_source, read_compiler_candidate_execution,
    render_compiler_candidate_execution, CompilerCandidateExecution,
    CompilerCandidateExecutionInput, COMPILER_CANDIDATE_EXECUTION_AUTHORITY,
    COMPILER_CANDIDATE_EXECUTION_FILE, COMPILER_CANDIDATE_EXECUTION_PROTOCOL,
    COMPILER_CANDIDATE_EXECUTION_ROLE, COMPILER_CANDIDATE_RUNNER_CONTRACT,
};
pub use compiler_candidate_fresh_source::{
    build_compiler_candidate_fresh_source_capability,
    parse_compiler_candidate_fresh_source_capability,
    parse_compiler_candidate_fresh_source_capability_from_source,
    render_compiler_candidate_fresh_source_capability,
    verify_compiler_candidate_fresh_source_capability, CompilerCandidateFreshSourceCapability,
    CompilerCandidateFreshSourceCapabilityInput, COMPILER_CANDIDATE_FRESH_SOURCE_ABI_CONTRACT,
    COMPILER_CANDIDATE_FRESH_SOURCE_ARGUMENT_CONTRACT, COMPILER_CANDIDATE_FRESH_SOURCE_AUTHORITY,
    COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE,
    COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL,
    COMPILER_CANDIDATE_FRESH_SOURCE_DRIVER_CONTRACT,
    COMPILER_CANDIDATE_FRESH_SOURCE_ENVIRONMENT_CONTRACT,
    COMPILER_CANDIDATE_FRESH_SOURCE_INPUT_CONTRACT,
    COMPILER_CANDIDATE_FRESH_SOURCE_NATIVE_CONTRACT, COMPILER_CANDIDATE_FRESH_SOURCE_VERDICT,
};
pub use compiler_candidate_fresh_source_result::{
    build_compiler_candidate_fresh_source_result, parse_compiler_candidate_fresh_source_result,
    parse_compiler_candidate_fresh_source_result_bytes,
    parse_compiler_candidate_fresh_source_result_from_source,
    render_compiler_candidate_fresh_source_result, CompilerCandidateFreshSourceResult,
    CompilerCandidateFreshSourceStage, COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE,
    COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_PROTOCOL, COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT,
    COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT,
};
pub use compiler_candidate_frontend_result::{
    build_compiler_candidate_frontend_result, parse_compiler_candidate_frontend_result,
    parse_compiler_candidate_frontend_result_bytes,
    parse_compiler_candidate_frontend_result_from_source,
    render_compiler_candidate_frontend_result, CompilerCandidateFrontendResult,
    COMPILER_CANDIDATE_FRONTEND_RESULT_FILE, COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL,
};
pub use compiler_candidate_nsld_input::{
    build_compiler_candidate_nsld_input, parse_compiler_candidate_nsld_input,
    parse_compiler_candidate_nsld_input_bytes, parse_compiler_candidate_nsld_input_from_source,
    render_compiler_candidate_nsld_input, CompilerCandidateNsldInput,
    COMPILER_CANDIDATE_NSLD_ENTRY_SYMBOL, COMPILER_CANDIDATE_NSLD_FUNCTION_CONTRACT,
    COMPILER_CANDIDATE_NSLD_GLM_CONTRACT, COMPILER_CANDIDATE_NSLD_INPUT_CONTRACT,
    COMPILER_CANDIDATE_NSLD_INPUT_FILE, COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL,
    COMPILER_CANDIDATE_NSLD_OPERATION_CONTRACT, COMPILER_CANDIDATE_NSLD_TARGET_CONTRACT,
    COMPILER_CANDIDATE_NSLD_TARGET_SELECTOR, COMPILER_CANDIDATE_NSLD_TIME_CONTRACT,
};
pub use compiler_candidate_nsld_materialization::{
    build_compiler_candidate_nsld_materialization_capability,
    parse_compiler_candidate_nsld_materialization_capability,
    parse_compiler_candidate_nsld_materialization_capability_from_source,
    render_compiler_candidate_nsld_materialization_capability,
    verify_compiler_candidate_nsld_materialization_capability,
    CompilerCandidateNsldMaterializationCapability,
    CompilerCandidateNsldMaterializationCapabilityInput,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_ARGUMENT_CONTRACT,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_AUTHORITY,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_DRIVER, COMPILER_CANDIDATE_NSLD_MATERIALIZATION_FILE,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_PROTOCOL,
    COMPILER_CANDIDATE_NSLD_MATERIALIZATION_VERDICT,
};
pub use compiler_candidate_preselection::{
    build_compiler_candidate_preselection, parse_compiler_candidate_preselection,
    parse_compiler_candidate_preselection_from_source, render_compiler_candidate_preselection,
    verify_compiler_candidate_preselection, CompilerCandidatePreselection,
    CompilerCandidatePreselectionInput, CompilerCandidatePreselectionVerificationInput,
    COMPILER_CANDIDATE_PRESELECTION_ACTION, COMPILER_CANDIDATE_PRESELECTION_AUTHORITY,
    COMPILER_CANDIDATE_PRESELECTION_FILE, COMPILER_CANDIDATE_PRESELECTION_PROTOCOL,
    COMPILER_CANDIDATE_PRESELECTION_PROVIDER_CONTRACT,
    COMPILER_CANDIDATE_PRESELECTION_SIGNATURE_CONTRACT, COMPILER_CANDIDATE_PRESELECTION_VERDICT,
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
pub use compiler_candidate_successor::{
    build_compiler_candidate_successor, parse_compiler_candidate_successor,
    parse_compiler_candidate_successor_from_source, render_compiler_candidate_successor,
    verify_compiler_candidate_successor, CompilerCandidateSuccessor,
    CompilerCandidateSuccessorInput, CompilerCandidateSuccessorVerificationInput,
    COMPILER_CANDIDATE_SUCCESSOR_ACTION, COMPILER_CANDIDATE_SUCCESSOR_AUTHORITY,
    COMPILER_CANDIDATE_SUCCESSOR_FILE, COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL,
    COMPILER_CANDIDATE_SUCCESSOR_RELATION_CONTRACT,
    COMPILER_CANDIDATE_SUCCESSOR_SIGNATURE_CONTRACT, COMPILER_CANDIDATE_SUCCESSOR_VERDICT,
};
pub use compiler_component_active_state::{
    build_compiler_component_active_state, parse_compiler_component_active_state,
    parse_compiler_component_active_state_from_source, render_compiler_component_active_state,
    select_compiler_component_active_target, verify_compiler_component_active_state,
    CompilerComponentActiveSelection, CompilerComponentActiveState, CompilerComponentActiveTarget,
    COMPILER_COMPONENT_ACTIVE_SELECTION_CONTRACT, COMPILER_COMPONENT_ACTIVE_SELECTOR,
    COMPILER_COMPONENT_ACTIVE_STATE_AUTHORITY, COMPILER_COMPONENT_ACTIVE_STATE_FILE,
    COMPILER_COMPONENT_ACTIVE_STATE_PROTOCOL, COMPILER_COMPONENT_ACTIVE_STATE_VERDICT,
    COMPILER_COMPONENT_ROLLBACK_SELECTOR,
};
pub use compiler_component_attestation::{
    build_compiler_component_attestation, parse_compiler_component_attestation,
    parse_compiler_component_attestation_from_source, read_compiler_component_attestation,
    render_compiler_component_attestation, verify_compiler_component_attestation,
    CompilerComponentAttestation, CompilerComponentAttestationInput,
    COMPILER_COMPONENT_ATTESTATION_AUTHORITY, COMPILER_COMPONENT_ATTESTATION_FILE,
    COMPILER_COMPONENT_ATTESTATION_PROTOCOL, COMPILER_COMPONENT_ATTESTATION_SIGNATURE_CONTRACT,
    COMPILER_COMPONENT_ATTESTATION_TRUST_SCOPE, COMPILER_COMPONENT_ATTESTATION_VERDICT,
};
pub use compiler_component_attestation_registry::{
    build_compiler_component_attester_trust_registry, compiler_component_attester_public_key_id,
    compiler_component_attester_trust_registry_sha256,
    parse_compiler_component_attester_trust_registry,
    parse_compiler_component_attester_trust_registry_from_source,
    render_compiler_component_attester_trust_registry, CompilerComponentAttesterTrustEntry,
    CompilerComponentAttesterTrustEntryInput, CompilerComponentAttesterTrustRegistry,
    COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_FILE,
    COMPILER_COMPONENT_ATTESTER_TRUST_REGISTRY_PROTOCOL,
};
pub use compiler_component_build::{
    build_compiler_component_build, parse_compiler_component_build,
    parse_compiler_component_build_from_source, promote_compiler_component_candidate,
    read_compiler_component_build, render_compiler_component_build,
    verify_compiler_component_build, verify_compiler_component_build_image, CompilerComponentBuild,
    CompilerComponentBuildInput, CompilerComponentCandidatePromotionInput,
    CompilerComponentDependency, CompilerComponentDependencyInput, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_BUILD_PROTOCOL, COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT,
    COMPILER_COMPONENT_DRIVER_CONTRACT, COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
};
pub use compiler_component_compile_dispatch::{
    build_compiler_component_compile_dispatch_receipt,
    parse_compiler_component_compile_dispatch_receipt,
    parse_compiler_component_compile_dispatch_receipt_from_source,
    render_compiler_component_compile_dispatch_receipt, CompilerComponentCompileDispatchReceipt,
    CompilerComponentCompileDispatchReceiptInput,
    COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT,
    COMPILER_COMPONENT_COMPILE_ARGUMENT_CONTRACT, COMPILER_COMPONENT_COMPILE_COMMAND,
    COMPILER_COMPONENT_COMPILE_DISPATCH_AUTHORITY,
    COMPILER_COMPONENT_COMPILE_DISPATCH_DRIVER_CONTRACT, COMPILER_COMPONENT_COMPILE_DISPATCH_FILE,
    COMPILER_COMPONENT_COMPILE_DISPATCH_PROTOCOL, COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT,
    COMPILER_COMPONENT_COMPILE_REQUEST_CONTRACT,
};
pub use compiler_component_diff::{
    build_compiler_component_differential, compare_compiler_component_paths,
    parse_compiler_component_differential, parse_compiler_component_differential_from_source,
    render_compiler_component_differential, CompilerComponentComparison,
    CompilerComponentDifferential, CompilerComponentEvidence, COMPILER_COMPONENT_DIFFERENTIAL_FILE,
    COMPILER_COMPONENT_DIFFERENTIAL_GATE_CONTRACT, COMPILER_COMPONENT_DIFFERENTIAL_PROTOCOL,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT,
};
pub use compiler_component_dispatch::{
    build_compiler_component_dispatch_receipt, parse_compiler_component_dispatch_receipt,
    parse_compiler_component_dispatch_receipt_from_source,
    render_compiler_component_dispatch_receipt, resolve_compiler_component_dispatch,
    CompilerComponentDispatchCandidate, CompilerComponentDispatchReceipt,
    CompilerComponentDispatchReceiptInput, CompilerComponentDispatchResolution,
    COMPILER_COMPONENT_DISPATCH_AUTHORITY, COMPILER_COMPONENT_DISPATCH_DRIVER_CONTRACT,
    COMPILER_COMPONENT_DISPATCH_FILE, COMPILER_COMPONENT_DISPATCH_INVENTORY_CONTRACT,
    COMPILER_COMPONENT_DISPATCH_PROTOCOL, COMPILER_COMPONENT_DISPATCH_REQUEST_ARGUMENT,
    COMPILER_COMPONENT_DISPATCH_REQUEST_CONTRACT, COMPILER_COMPONENT_DISPATCH_VERDICT,
};
pub use compiler_component_replacement::{
    build_compiler_component_replacement_authorization,
    parse_compiler_component_replacement_authorization,
    parse_compiler_component_replacement_authorization_from_source,
    render_compiler_component_replacement_authorization,
    verify_compiler_component_replacement_authorization, CompilerComponentReplacementAuthorization,
    CompilerComponentReplacementAuthorizationInput, CompilerComponentReplacementVerificationInput,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_ACTION,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_AUTHORITY,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_PROTOCOL,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_SIGNATURE_CONTRACT,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_VERDICT,
};
pub use compiler_component_replacement_registry::{
    build_compiler_component_replacement_authorizer_registry,
    compiler_component_replacement_authorizer_public_key_id,
    compiler_component_replacement_authorizer_registry_sha256,
    parse_compiler_component_replacement_authorizer_registry,
    parse_compiler_component_replacement_authorizer_registry_from_source,
    render_compiler_component_replacement_authorizer_registry,
    CompilerComponentReplacementAuthorizerEntry, CompilerComponentReplacementAuthorizerEntryInput,
    CompilerComponentReplacementAuthorizerRegistry,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_FILE,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_REGISTRY_PROTOCOL,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORIZER_TRUST_SCOPE,
};
pub use compiler_component_representation_diff::{
    build_compiler_component_representation_differential,
    compare_compiler_component_representation_paths,
    parse_compiler_component_representation_differential,
    parse_compiler_component_representation_differential_from_source,
    read_compiler_component_representation_differential,
    render_compiler_component_representation_differential,
    CompilerComponentRepresentationComparison, CompilerComponentRepresentationDifferential,
    CompilerComponentRepresentationDifferentialInput,
    COMPILER_COMPONENT_REPRESENTATION_COMPARISON_CONTRACT,
    COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_AUTHORITY,
    COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE,
    COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_PROTOCOL,
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
pub use compiler_component_transition::{
    build_compiler_component_transition, parse_compiler_component_transition,
    parse_compiler_component_transition_from_source, render_compiler_component_transition,
    select_compiler_component_transition_target, verify_compiler_component_transition,
    CompilerComponentTransition, CompilerComponentTransitionInput,
    CompilerComponentTransitionSelection, CompilerComponentTransitionTarget,
    CompilerComponentTransitionVerificationInput, COMPILER_COMPONENT_TRANSITION_ACTION,
    COMPILER_COMPONENT_TRANSITION_AUTHORITY, COMPILER_COMPONENT_TRANSITION_FILE,
    COMPILER_COMPONENT_TRANSITION_PROTOCOL, COMPILER_COMPONENT_TRANSITION_SIGNATURE_CONTRACT,
    COMPILER_COMPONENT_TRANSITION_VERDICT,
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
pub use compiler_stage_handoff_v2::{
    build_compiler_stage_handoff_v2, parse_compiler_stage_handoff_v2,
    parse_compiler_stage_handoff_v2_from_source, read_compiler_stage_handoff_v2,
    render_compiler_stage_handoff_v2, verify_compiler_stage_handoff_v2, CompilerStageHandoffV2,
    CompilerStageHandoffV2Input, CompilerStageSelectionRecord, COMPILER_STAGE_HANDOFF_V2_AUTHORITY,
    COMPILER_STAGE_HANDOFF_V2_FILE, COMPILER_STAGE_HANDOFF_V2_PROTOCOL,
    COMPILER_STAGE_HANDOFF_V2_SELECTION_CONTRACT, COMPILER_STAGE_HANDOFF_V2_VERDICT,
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
    COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT, COMPILER_STAGE_TRANSFORMATION_AUTHORITY,
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
    compiler_token_first_page_identity, compiler_token_page_identity, decode_compiler_token_stream,
    CompilerTokenDecodeSummary, CompilerTokenPageIdentity, COMPILER_TOKEN_DECODER_CONTRACT,
    COMPILER_TOKEN_DECODER_FOLD_MODULUS, COMPILER_TOKEN_DECODER_MAX_BYTES,
    COMPILER_TOKEN_DECODER_MAX_RECORDS, COMPILER_TOKEN_DECODER_SEMANTIC_SEED,
    COMPILER_TOKEN_PAGE_CANONICAL_BYTES, COMPILER_TOKEN_PAGE_IDENTITY_RADIX,
    COMPILER_TOKEN_PAGE_PAYLOAD_BYTES, COMPILER_TOKEN_PAGE_RECORDS, COMPILER_TOKEN_STREAM_PROTOCOL,
};
pub use compiler_token_pagination::{
    compiler_token_page_chain_fold, compiler_token_page_hash_step,
    compiler_token_pagination_identity, CompilerTokenPaginationIdentity,
    CompilerTokenPaginationPage, COMPILER_TOKEN_PAGE_CHAIN_RADIX, COMPILER_TOKEN_PAGE_CHAIN_SEED,
    COMPILER_TOKEN_PAGE_HASH_RADIX, COMPILER_TOKEN_PAGE_HASH_SEED,
    COMPILER_TOKEN_PAGINATION_CONTRACT, COMPILER_TOKEN_PAGINATION_PAGE_BYTES,
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
