pub(crate) use super::container_pipeline_verify::nsld_verify_container_report;

use super::{assembly::nsld_section_manifest_report, container, fnv1a64_hex, toml};
use std::{
    fs,
    path::{Path, PathBuf},
};

const NSLD_CONTAINER_MAGIC: &str = "NUISNSLD";
const NSLD_CONTAINER_VERSION: usize = 1;

pub(crate) fn nsld_container_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> container::NsldContainerPlanReport {
    let section_manifest = nsld_section_manifest_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container")
        .display()
        .to_string();
    let container_layout_hash = container::layout_hash(
        NSLD_CONTAINER_MAGIC,
        NSLD_CONTAINER_VERSION,
        section_manifest.section_count,
        &section_manifest.section_table_hash,
        &output_path,
        fnv1a64_hex,
    );
    container::NsldContainerPlanReport {
        manifest: manifest.display().to_string(),
        ready: section_manifest.ready,
        container_magic: NSLD_CONTAINER_MAGIC.to_owned(),
        container_version: NSLD_CONTAINER_VERSION,
        section_count: section_manifest.section_count,
        section_table_hash: section_manifest.section_table_hash,
        container_layout_hash,
        output_path,
        sections: section_manifest.sections,
        blockers: section_manifest.blockers,
    }
}

pub(crate) fn nsld_emit_container_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<container::NsldContainerPlanEmitReport, String> {
    let report = nsld_container_plan_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container-plan.toml");
    fs::write(&output_path, container::render_container_plan_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld container plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(container::NsldContainerPlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        ready: report.ready,
        container_layout_hash: report.container_layout_hash,
        section_count: report.section_count,
    })
}

pub(crate) fn nsld_verify_container_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> container::NsldContainerPlanVerifyReport {
    let expected_report = nsld_container_plan_report(manifest, plan);
    let expected = container::render_container_plan_toml(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.container-plan.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_container_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_container_layout_hash, actual_section_count) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "container_layout_hash"),
            toml::usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("container-plan-content-mismatch".to_owned());
        }
        if actual_container_layout_hash.as_deref()
            != Some(expected_report.container_layout_hash.as_str())
        {
            issues.push(format!(
                "container_layout_hash mismatch: expected {}, found {}",
                expected_report.container_layout_hash,
                actual_container_layout_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    container::NsldContainerPlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_container_layout_hash: expected_report.container_layout_hash,
        expected_section_count: expected_report.section_count,
        actual_container_layout_hash,
        actual_section_count,
        issues,
    }
}

pub(crate) fn nsld_container_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> container::NsldContainerReport {
    let container_plan = nsld_container_plan_report(manifest, plan);
    let sections = container::section_entries(&container_plan.sections, fnv1a64_hex);
    let container_section_table_hash =
        container::container_section_table_hash(&sections, fnv1a64_hex);
    let loader_entry_kind = "lifecycle-bootstrap".to_owned();
    let loader_entry_symbol = plan.lifecycle.bootstrap_entry.clone();
    let loader_entry_section_id = sections
        .iter()
        .find(|section| section.section_kind == "compiled-artifact")
        .map(|section| section.section_id.clone())
        .unwrap_or_else(|| "missing".to_owned());
    let mut loader_symbols = container::loader_symbols(
        &loader_entry_kind,
        &loader_entry_symbol,
        &loader_entry_section_id,
        &sections,
    );
    loader_symbols.extend(container::hetero_loader_symbols(
        &plan.hetero_calculate.nodes,
        &sections,
        loader_symbols.len(),
    ));
    let loader_symbol_table_hash =
        container::loader_symbol_table_hash(&loader_symbols, fnv1a64_hex);
    let relocations = container::relocations(&loader_symbols);
    let relocation_table_hash = container::relocation_table_hash(&relocations, fnv1a64_hex);
    let external_imports = container::external_imports(plan);
    let external_import_table_hash =
        container::external_import_table_hash(&external_imports, fnv1a64_hex);
    let metadata_table_hash = container::metadata_table_hash(
        &container_section_table_hash,
        &loader_symbol_table_hash,
        &relocation_table_hash,
        &external_import_table_hash,
        fnv1a64_hex,
    );
    let loader_blockers = container::loader_blockers(&external_imports, &container_plan.blockers);
    let loader_readiness = if !container_plan.ready || !container_plan.blockers.is_empty() {
        "blocked"
    } else if external_imports
        .iter()
        .any(|external_import| external_import.required)
    {
        "host-assisted"
    } else {
        "self-contained"
    }
    .to_owned();
    let payload_size_bytes = container::payload_size(&sections);
    let payload_hash = container::payload_hash(&sections, fnv1a64_hex);
    let container_hash = container::file_hash(
        &container_plan,
        &sections,
        &loader_entry_kind,
        &loader_entry_symbol,
        &loader_entry_section_id,
        &loader_symbols,
        &relocations,
        &external_imports,
        &loader_readiness,
        &loader_blockers,
        payload_size_bytes,
        &payload_hash,
        fnv1a64_hex,
    );
    container::NsldContainerReport {
        manifest: manifest.display().to_string(),
        ready: container_plan.ready,
        container_magic: container_plan.container_magic,
        container_version: container_plan.container_version,
        metadata_table_hash,
        container_layout_hash: container_plan.container_layout_hash,
        container_hash,
        loader_readiness,
        loader_blockers,
        loader_entry_kind,
        loader_entry_symbol,
        loader_entry_section_id,
        loader_symbol_table_hash,
        loader_symbols,
        relocation_table_hash,
        relocations,
        external_import_table_hash,
        external_imports,
        payload_size_bytes,
        payload_hash,
        payload_path: format!("{}.payload", container_plan.output_path),
        output_path: container_plan.output_path,
        section_count: container_plan.section_count,
        container_section_table_hash,
        sections,
        blockers: container_plan.blockers,
    }
}

pub(crate) fn nsld_emit_container_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<container::NsldContainerEmitReport, String> {
    let report = nsld_container_report(manifest, plan);
    let output_path = PathBuf::from(&report.output_path);
    let payload_path = PathBuf::from(&report.payload_path);
    fs::write(&payload_path, container::payload_bytes(&report.sections)).map_err(|error| {
        format!(
            "failed to write nsld container payload `{}`: {error}",
            payload_path.display()
        )
    })?;
    fs::write(&output_path, container::render_container_toml(&report)).map_err(|error| {
        format!(
            "failed to write nsld container `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(container::NsldContainerEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        payload_path: payload_path.display().to_string(),
        ready: report.ready,
        metadata_table_hash: report.metadata_table_hash,
        container_layout_hash: report.container_layout_hash,
        container_hash: report.container_hash,
        payload_size_bytes: report.payload_size_bytes,
        payload_hash: report.payload_hash,
        section_count: report.section_count,
    })
}
