pub use crate::aot_artifact::{
    decode_nuis_compiled_artifact_binary, decode_nuis_executable_envelope_binary,
    encode_nuis_compiled_artifact_binary, encode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_executable_envelope_binary, inspect_nuis_compiled_artifact_container,
    parse_nuis_compiled_artifact, parse_nuis_executable_envelope,
    parse_nuis_executable_envelope_from_source, render_nuis_executable_envelope,
    validate_nuis_compiled_artifact_layout, write_nuis_compiled_artifact,
    write_nuis_executable_envelope, NuisCompiledArtifactContainerInspect,
    NuisCompiledArtifactLoweringUnitInspect,
};
#[cfg(test)]
use crate::aot_c_shim_source::render_c_shim_source as c_shim_source;
pub use crate::aot_compile_driver::{
    compile_artifacts_for_output_dir, compile_artifacts_for_output_dir_with_packaging_mode,
    write_and_link,
};
pub use crate::aot_compiled_artifact_verify::verify_nuis_compiled_artifact;
pub use crate::aot_cpu_target::{
    host_cpu_build_target, resolve_cpu_build_target, resolve_cpu_build_target_from_abi,
    resolve_cpu_build_target_from_project_abi, resolve_cpu_build_target_from_target,
    CpuBuildTarget,
};
#[cfg(test)]
use crate::aot_domain_payload_blob::decode_domain_build_unit_payload_blob;
#[cfg(test)]
use crate::aot_domain_render::{
    render_domain_build_unit_backend_stub, render_domain_build_unit_bridge_plan,
    render_domain_build_unit_host_bridge_stub, render_domain_build_unit_lowering_plan,
};
#[cfg(test)]
use crate::aot_kernel_sidecar::render_domain_build_unit_kernel_ir_sidecar;
#[cfg(test)]
use crate::aot_lifecycle::build_nuis_lifecycle_contract;
pub use crate::aot_manifest_relocate::render_relocated_unpacked_build_manifest;
pub use crate::aot_manifest_types::{
    BuildManifestCacheInfo, BuildManifestContext, BuildManifestDocIndexInfo,
    BuildManifestProjectInfo, CompileArtifacts,
};
pub use crate::aot_manifest_verify::verify_build_manifest;
pub use crate::aot_manifest_writer::write_build_manifest;
#[cfg(test)]
use crate::aot_network_sidecar::render_domain_build_unit_network_ir_sidecar;
#[cfg(test)]
use crate::aot_project_metadata_verify::project_metadata_summary_mismatch_error;
#[cfg(test)]
use crate::aot_shader_sidecar::render_domain_build_unit_shader_ir_sidecar;
pub use crate::aot_verify_report::{BuildManifestVerifyReport, NuisCompiledArtifactVerifyReport};

pub use nuis_artifact::{
    BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob, NuisCompiledArtifact,
    NuisExecutableEnvelope, NuisLifecycleContract,
};

#[cfg(test)]
#[path = "aot_tests.rs"]
mod tests;
