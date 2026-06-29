use std::path::Path;

use nuis_artifact::{BuildManifestDomainBuildUnit, NuisExecutableEnvelope, NuisLifecycleContract};

use crate::aot_artifact_hash::{
    append_artifact_hash_manifest_sections, append_artifacts_manifest_section,
};
use crate::aot_domain_index_render::append_build_manifest_domain_index_sections;
use crate::aot_domain_unit_render::append_domain_build_unit_manifest_sections;
use crate::aot_manifest_artifacts::BuildManifestArtifactSet;
use crate::aot_manifest_domain_model::BuildManifestExecutionContract;
use crate::aot_manifest_execution_render::append_execution_contract_manifest_sections;
use crate::aot_manifest_header_render::{
    append_manifest_header_sections, BuildManifestHeaderRenderInput,
};
use crate::aot_manifest_project_render::append_project_manifest_section;
use crate::aot_manifest_types::{BuildManifestContext, BuildManifestProjectInfo};
use crate::aot_vcs_info::VcsInfo;

pub(crate) struct BuildManifestRenderInput<'a> {
    pub(crate) generated_at_unix: u64,
    pub(crate) packaging_mode: &'a str,
    pub(crate) engine_version: &'a str,
    pub(crate) engine_profile: &'a str,
    pub(crate) vcs: &'a VcsInfo,
    pub(crate) loaded_nustar: &'a [String],
    pub(crate) envelope_path: &'a Path,
    pub(crate) envelope: &'a NuisExecutableEnvelope,
    pub(crate) artifact_path: &'a Path,
    pub(crate) artifact_binary_name: &'a str,
    pub(crate) artifact_binary_bytes: usize,
    pub(crate) lifecycle: &'a NuisLifecycleContract,
    pub(crate) artifact_set: &'a BuildManifestArtifactSet,
    pub(crate) execution_contracts: &'a [BuildManifestExecutionContract],
    pub(crate) domain_build_units: &'a [BuildManifestDomainBuildUnit],
    pub(crate) project: Option<&'a BuildManifestProjectInfo>,
}

pub(crate) fn render_build_manifest_source(
    context: &BuildManifestContext,
    input: BuildManifestRenderInput<'_>,
) -> Result<String, String> {
    let mut out = String::new();
    append_manifest_header_sections(
        &mut out,
        context,
        BuildManifestHeaderRenderInput {
            generated_at_unix: input.generated_at_unix,
            packaging_mode: input.packaging_mode,
            engine_version: input.engine_version,
            engine_profile: input.engine_profile,
            vcs: input.vcs,
            loaded_nustar: input.loaded_nustar,
            envelope_path: input.envelope_path,
            envelope: input.envelope,
            artifact_path: input.artifact_path,
            artifact_binary_name: input.artifact_binary_name,
            artifact_binary_bytes: input.artifact_binary_bytes,
            lifecycle: input.lifecycle,
        },
    );
    append_artifacts_manifest_section(&mut out, &input.artifact_set.artifacts);

    append_build_manifest_domain_index_sections(
        &mut out,
        input.artifact_set.bridge_registry_path.as_deref(),
        input.artifact_set.bridge_registry_inline.as_deref(),
        input.artifact_set.host_bridge_plan_index_path.as_deref(),
        input.artifact_set.host_bridge_plan_index_inline.as_deref(),
        input.artifact_set.lowering_plan_index_path.as_deref(),
        input.artifact_set.lowering_plan_index_inline.as_deref(),
        input.domain_build_units,
    );

    append_artifact_hash_manifest_sections(&mut out, &input.artifact_set.artifacts)?;

    append_execution_contract_manifest_sections(&mut out, input.execution_contracts);
    append_domain_build_unit_manifest_sections(&mut out, input.domain_build_units);

    if let Some(project) = input.project {
        append_project_manifest_section(&mut out, project);
    }

    Ok(out)
}
