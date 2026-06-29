use std::path::{Path, PathBuf};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_artifact_writer::write_domain_build_unit_stubs;
use crate::aot_domain_index_render::{
    render_domain_bridge_registry, render_domain_lowering_plan_index,
    render_host_bridge_plan_index, write_domain_bridge_registry, write_domain_lowering_plan_index,
    write_host_bridge_plan_index,
};
use crate::aot_manifest_types::CompileArtifacts;

pub(crate) struct BuildManifestArtifactSet {
    pub(crate) artifacts: Vec<(String, PathBuf)>,
    pub(crate) bridge_registry_path: Option<PathBuf>,
    pub(crate) bridge_registry_inline: Option<String>,
    pub(crate) host_bridge_plan_index_path: Option<PathBuf>,
    pub(crate) host_bridge_plan_index_inline: Option<String>,
    pub(crate) lowering_plan_index_path: Option<PathBuf>,
    pub(crate) lowering_plan_index_inline: Option<String>,
}

pub(crate) fn prepare_build_manifest_artifacts(
    output_dir: &Path,
    written: &CompileArtifacts,
    domain_build_units: &mut [BuildManifestDomainBuildUnit],
) -> Result<BuildManifestArtifactSet, String> {
    let mut artifacts = vec![
        ("ast".to_owned(), PathBuf::from(&written.ast_path)),
        ("nir".to_owned(), PathBuf::from(&written.nir_path)),
        ("yir".to_owned(), PathBuf::from(&written.yir_path)),
        ("llvm_ir".to_owned(), PathBuf::from(&written.llvm_ir_path)),
        ("binary".to_owned(), PathBuf::from(&written.binary_path)),
    ];
    artifacts.extend(write_domain_build_unit_stubs(
        output_dir,
        domain_build_units,
    )?);

    let hetero_units = domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let bridge_registry_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_domain_bridge_registry(&hetero_units))
    };
    let host_bridge_plan_index_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_host_bridge_plan_index(&hetero_units))
    };
    let lowering_plan_index_inline = if hetero_units.is_empty() {
        None
    } else {
        Some(render_domain_lowering_plan_index(&hetero_units))
    };

    let bridge_registry_path = write_domain_bridge_registry(output_dir, domain_build_units)?;
    if let Some(bridge_registry_path) = &bridge_registry_path {
        artifacts.push((
            "domain_bridge_registry".to_owned(),
            bridge_registry_path.clone(),
        ));
    }
    let host_bridge_plan_index_path = write_host_bridge_plan_index(output_dir, domain_build_units)?;
    if let Some(host_bridge_plan_index_path) = &host_bridge_plan_index_path {
        artifacts.push((
            "host_bridge_plan_index".to_owned(),
            host_bridge_plan_index_path.clone(),
        ));
    }
    let lowering_plan_index_path =
        write_domain_lowering_plan_index(output_dir, domain_build_units)?;
    if let Some(lowering_plan_index_path) = &lowering_plan_index_path {
        artifacts.push((
            "domain_lowering_plan_index".to_owned(),
            lowering_plan_index_path.clone(),
        ));
    }

    Ok(BuildManifestArtifactSet {
        artifacts,
        bridge_registry_path,
        bridge_registry_inline,
        host_bridge_plan_index_path,
        host_bridge_plan_index_inline,
        lowering_plan_index_path,
        lowering_plan_index_inline,
    })
}
