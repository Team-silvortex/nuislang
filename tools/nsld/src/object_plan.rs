use super::{
    assembly::nsld_section_manifest_report,
    container_verify::{self, TomlFieldKind},
    fnv1a64_hex,
    reports::{
        NsldObjectEmitReport, NsldObjectPlanEmitReport, NsldObjectPlanReport,
        NsldObjectPlanVerifyReport, NsldObjectRelocationSeedDiagnostic,
        NsldObjectSectionDiagnostic, NsldObjectWriterReadinessReport,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectPlanReport {
    let section_manifest = nsld_section_manifest_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-plan.toml");
    let source_container_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container")
        .display()
        .to_string();
    let source_payload_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container.payload")
        .display()
        .to_string();
    let unsupported_features = vec![
        "object-byte-emitter".to_owned(),
        "native-relocation-applier".to_owned(),
    ];
    let mut blockers = section_manifest.blockers.clone();
    blockers.extend(
        unsupported_features
            .iter()
            .map(|feature| format!("{feature}:not-implemented")),
    );
    let object_sections = object_section_layout(&section_manifest.sections);
    let relocation_seeds = object_relocation_seeds(&object_sections);
    let object_layout_hash = nsld_object_layout_hash(&object_sections);
    let relocation_seed_table_hash = nsld_relocation_seed_table_hash(&relocation_seeds);
    let object_plan_hash = nsld_object_plan_hash(
        &plan.cpu_target.machine_arch,
        &plan.cpu_target.machine_os,
        &plan.cpu_target.object_format,
        &section_manifest.section_table_hash,
        &object_layout_hash,
        &relocation_seed_table_hash,
        &source_container_path,
        &source_payload_path,
        &object_sections,
        &relocation_seeds,
        &blockers,
    );

    NsldObjectPlanReport {
        manifest: manifest.display().to_string(),
        ready: section_manifest.ready && blockers.is_empty(),
        target_arch: plan.cpu_target.machine_arch.clone(),
        target_os: plan.cpu_target.machine_os.clone(),
        object_format: plan.cpu_target.object_format.clone(),
        calling_abi: plan.cpu_target.calling_abi.clone(),
        clang_target: plan.cpu_target.clang_target.clone(),
        output_path: output_path.display().to_string(),
        source_container_path,
        source_payload_path,
        section_count: section_manifest.section_count,
        section_table_hash: section_manifest.section_table_hash,
        object_plan_hash,
        object_layout_hash,
        relocation_seed_count: relocation_seeds.len(),
        relocation_seed_table_hash,
        writer_target_id: writer_target_id(
            &plan.cpu_target.machine_arch,
            &plan.cpu_target.machine_os,
            &plan.cpu_target.object_format,
        ),
        writer_status: "blocked".to_owned(),
        unsupported_features,
        emission_status: "plan-only".to_owned(),
        object_sections,
        relocation_seeds,
        blockers,
    }
}

pub(crate) fn nsld_emit_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectPlanEmitReport, String> {
    let report = nsld_object_plan_report(manifest, plan);
    fs::write(&report.output_path, toml::render_object_plan(&report)).map_err(|error| {
        format!(
            "failed to write nsld object plan `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldObjectPlanEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        ready: report.ready,
        object_plan_hash: report.object_plan_hash,
        section_count: report.section_count,
    })
}

pub(crate) fn nsld_verify_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectPlanVerifyReport {
    let expected_report = nsld_object_plan_report(manifest, plan);
    let expected = toml::render_object_plan(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-plan.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_object_plan_hash, actual_section_count) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-plan-content-mismatch".to_owned());
        }
        issues.extend(object_section_table_field_issues(&actual));
        issues.extend(object_section_table_mismatch_issues(
            &expected_report.object_sections,
            &object_section_entries(&actual),
        ));
        issues.extend(relocation_seed_table_field_issues(&actual));
        issues.extend(relocation_seed_table_mismatch_issues(
            &expected_report.relocation_seeds,
            &relocation_seed_entries(&actual),
        ));
        if actual_object_plan_hash.as_deref() != Some(expected_report.object_plan_hash.as_str()) {
            issues.push(format!(
                "object_plan_hash mismatch: expected {}, found {}",
                expected_report.object_plan_hash,
                actual_object_plan_hash
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

    NsldObjectPlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_object_plan_hash: expected_report.object_plan_hash,
        expected_section_count: expected_report.section_count,
        actual_object_plan_hash,
        actual_section_count,
        issues,
    }
}

pub(crate) fn nsld_object_writer_readiness_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectWriterReadinessReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    NsldObjectWriterReadinessReport {
        manifest: object_plan.manifest,
        writer_target_id: object_plan.writer_target_id,
        writer_status: object_plan.writer_status,
        object_plan_hash: object_plan.object_plan_hash,
        section_count: object_plan.section_count,
        can_emit_object: object_plan.ready
            && object_plan.unsupported_features.is_empty()
            && object_plan.blockers.is_empty(),
        unsupported_features: object_plan.unsupported_features,
        blockers: object_plan.blockers,
    }
}

pub(crate) fn nsld_emit_object_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectEmitReport, String> {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let readiness = nsld_object_writer_readiness_report(manifest, plan);
    let blocked_report_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object.blocked.toml");
    let report = NsldObjectEmitReport {
        manifest: readiness.manifest,
        output_path: PathBuf::from(&plan.output_dir)
            .join(format!("nuis.nsld.{}", object_plan.object_format))
            .display()
            .to_string(),
        blocked_report_path: blocked_report_path.display().to_string(),
        writer_target_id: readiness.writer_target_id,
        object_plan_hash: readiness.object_plan_hash,
        emitted: false,
        can_emit_object: readiness.can_emit_object,
        blockers: readiness.blockers,
    };
    if !report.emitted {
        fs::write(
            &blocked_report_path,
            toml::render_object_emit_blocked(&report),
        )
        .map_err(|error| {
            format!(
                "failed to write nsld blocked object emit report `{}`: {error}",
                blocked_report_path.display()
            )
        })?;
    }
    Ok(report)
}

fn nsld_object_plan_hash(
    target_arch: &str,
    target_os: &str,
    object_format: &str,
    section_table_hash: &str,
    object_layout_hash: &str,
    relocation_seed_table_hash: &str,
    source_container_path: &str,
    source_payload_path: &str,
    object_sections: &[NsldObjectSectionDiagnostic],
    relocation_seeds: &[NsldObjectRelocationSeedDiagnostic],
    blockers: &[String],
) -> String {
    let section_material = object_sections
        .iter()
        .map(|section| {
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                section.order_index,
                section.source_section_id,
                section.source_section_kind,
                section.object_section_name,
                section.object_section_role,
                section.source_size_bytes,
                section.payload_offset_seed,
                section.file_offset_seed,
                section.file_size_seed,
                section.alignment
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let relocation_material = relocation_seeds
        .iter()
        .map(|seed| {
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}",
                seed.order_index,
                seed.relocation_seed_id,
                seed.relocation_seed_kind,
                seed.source_section_id,
                seed.source_offset_seed,
                seed.target_symbol,
                seed.addend,
                seed.native_relocation_ready
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let material = format!(
        "target_arch={target_arch}\ntarget_os={target_os}\nobject_format={object_format}\nsection_table_hash={section_table_hash}\nobject_layout_hash={object_layout_hash}\nrelocation_seed_table_hash={relocation_seed_table_hash}\nsource_container_path={source_container_path}\nsource_payload_path={source_payload_path}\nobject_sections={section_material}\nrelocation_seeds={relocation_material}\nblockers={}\n",
        blockers.join("|")
    );
    fnv1a64_hex(material.as_bytes())
}

fn nsld_object_layout_hash(object_sections: &[NsldObjectSectionDiagnostic]) -> String {
    let mut material = String::new();
    for section in object_sections {
        material.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            section.order_index,
            section.source_section_id,
            section.object_section_name,
            section.source_size_bytes,
            section.payload_offset_seed,
            section.file_offset_seed,
            section.file_size_seed,
            section.alignment
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

fn nsld_relocation_seed_table_hash(
    relocation_seeds: &[NsldObjectRelocationSeedDiagnostic],
) -> String {
    let mut material = String::new();
    for seed in relocation_seeds {
        material.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            seed.order_index,
            seed.relocation_seed_id,
            seed.relocation_seed_kind,
            seed.source_section_id,
            seed.source_offset_seed,
            seed.target_symbol,
            seed.addend,
            seed.native_relocation_ready
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

fn object_section_table_field_issues(source: &str) -> Vec<String> {
    let mut issues = container_verify::table_field_issues(
        source,
        "object_section",
        "object_section",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("source_section_id", TomlFieldKind::String),
            ("source_section_kind", TomlFieldKind::String),
            ("object_section_name", TomlFieldKind::String),
            ("object_section_role", TomlFieldKind::String),
            ("source_path", TomlFieldKind::String),
            ("source_hash", TomlFieldKind::String),
            ("source_size_bytes", TomlFieldKind::Usize),
            ("payload_offset_seed", TomlFieldKind::Usize),
            ("file_offset_seed", TomlFieldKind::Usize),
            ("file_size_seed", TomlFieldKind::Usize),
            ("alignment", TomlFieldKind::Usize),
            ("required", TomlFieldKind::Bool),
        ],
    );
    issues.extend(container_verify::table_field_issues(
        &format!("[[object_plan_header]]\n{source}"),
        "object_plan_header",
        "object_plan_header",
        &[
            ("writer_target_id", TomlFieldKind::String),
            ("writer_status", TomlFieldKind::String),
            ("unsupported_features", TomlFieldKind::Array),
        ],
    ));
    issues
}

fn relocation_seed_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "object_relocation_seed",
        "object_relocation_seed",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("relocation_seed_id", TomlFieldKind::String),
            ("relocation_seed_kind", TomlFieldKind::String),
            ("source_section_id", TomlFieldKind::String),
            ("source_offset_seed", TomlFieldKind::Usize),
            ("target_symbol", TomlFieldKind::String),
            ("addend", TomlFieldKind::Isize),
            ("native_relocation_ready", TomlFieldKind::Bool),
        ],
    )
}

fn writer_target_id(machine_arch: &str, machine_os: &str, object_format: &str) -> String {
    format!("{machine_arch}-{machine_os}-{object_format}")
}

fn object_section_entries(source: &str) -> Vec<NsldObjectSectionDiagnostic> {
    toml_table_blocks(source, "object_section")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectSectionDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_section_kind: toml_block_string_value(&block, "source_section_kind")?,
                object_section_name: toml_block_string_value(&block, "object_section_name")?,
                object_section_role: toml_block_string_value(&block, "object_section_role")?,
                source_path: toml_block_string_value(&block, "source_path")?,
                source_hash: toml_block_string_value(&block, "source_hash")?,
                source_size_bytes: toml_block_usize_value(&block, "source_size_bytes")?,
                payload_offset_seed: toml_block_usize_value(&block, "payload_offset_seed")?,
                file_offset_seed: toml_block_usize_value(&block, "file_offset_seed")?,
                file_size_seed: toml_block_usize_value(&block, "file_size_seed")?,
                alignment: toml_block_usize_value(&block, "alignment")?,
                required: toml_block_bool_value(&block, "required")?,
            })
        })
        .collect()
}

fn relocation_seed_entries(source: &str) -> Vec<NsldObjectRelocationSeedDiagnostic> {
    toml_table_blocks(source, "object_relocation_seed")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectRelocationSeedDiagnostic {
                order_index: toml_block_usize_value(&block, "order_index")?,
                relocation_seed_id: toml_block_string_value(&block, "relocation_seed_id")?,
                relocation_seed_kind: toml_block_string_value(&block, "relocation_seed_kind")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_offset_seed: toml_block_usize_value(&block, "source_offset_seed")?,
                target_symbol: toml_block_string_value(&block, "target_symbol")?,
                addend: toml_block_isize_value(&block, "addend")?,
                native_relocation_ready: toml_block_bool_value(&block, "native_relocation_ready")?,
            })
        })
        .collect()
}

fn object_section_table_mismatch_issues(
    expected: &[NsldObjectSectionDiagnostic],
    actual: &[NsldObjectSectionDiagnostic],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "object_section_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("object_section[{index}] missing"));
            continue;
        };
        push_object_section_mismatch(
            &mut issues,
            index,
            "order_index",
            expected_entry.order_index,
            actual_entry.order_index,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_section_id",
            &expected_entry.source_section_id,
            &actual_entry.source_section_id,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_section_kind",
            &expected_entry.source_section_kind,
            &actual_entry.source_section_kind,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "object_section_name",
            &expected_entry.object_section_name,
            &actual_entry.object_section_name,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "object_section_role",
            &expected_entry.object_section_role,
            &actual_entry.object_section_role,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_path",
            &expected_entry.source_path,
            &actual_entry.source_path,
        );
        push_object_section_string_mismatch(
            &mut issues,
            index,
            "source_hash",
            &expected_entry.source_hash,
            &actual_entry.source_hash,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "source_size_bytes",
            expected_entry.source_size_bytes,
            actual_entry.source_size_bytes,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "payload_offset_seed",
            expected_entry.payload_offset_seed,
            actual_entry.payload_offset_seed,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "file_offset_seed",
            expected_entry.file_offset_seed,
            actual_entry.file_offset_seed,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "file_size_seed",
            expected_entry.file_size_seed,
            actual_entry.file_size_seed,
        );
        push_object_section_mismatch(
            &mut issues,
            index,
            "alignment",
            expected_entry.alignment,
            actual_entry.alignment,
        );
        if actual_entry.required != expected_entry.required {
            issues.push(format!(
                "object_section[{index}].required mismatch: expected {}, found {}",
                expected_entry.required, actual_entry.required
            ));
        }
    }
    issues
}

fn relocation_seed_table_mismatch_issues(
    expected: &[NsldObjectRelocationSeedDiagnostic],
    actual: &[NsldObjectRelocationSeedDiagnostic],
) -> Vec<String> {
    let mut issues = Vec::new();
    if actual.len() != expected.len() {
        issues.push(format!(
            "object_relocation_seed_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_entry) in expected.iter().enumerate() {
        let Some(actual_entry) = actual.get(index) else {
            issues.push(format!("object_relocation_seed[{index}] missing"));
            continue;
        };
        push_object_relocation_seed_mismatch(
            &mut issues,
            index,
            "order_index",
            expected_entry.order_index,
            actual_entry.order_index,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "relocation_seed_id",
            &expected_entry.relocation_seed_id,
            &actual_entry.relocation_seed_id,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "relocation_seed_kind",
            &expected_entry.relocation_seed_kind,
            &actual_entry.relocation_seed_kind,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "source_section_id",
            &expected_entry.source_section_id,
            &actual_entry.source_section_id,
        );
        push_object_relocation_seed_mismatch(
            &mut issues,
            index,
            "source_offset_seed",
            expected_entry.source_offset_seed,
            actual_entry.source_offset_seed,
        );
        push_object_relocation_seed_string_mismatch(
            &mut issues,
            index,
            "target_symbol",
            &expected_entry.target_symbol,
            &actual_entry.target_symbol,
        );
        if actual_entry.addend != expected_entry.addend {
            issues.push(format!(
                "object_relocation_seed[{index}].addend mismatch: expected {}, found {}",
                expected_entry.addend, actual_entry.addend
            ));
        }
        if actual_entry.native_relocation_ready != expected_entry.native_relocation_ready {
            issues.push(format!(
                "object_relocation_seed[{index}].native_relocation_ready mismatch: expected {}, found {}",
                expected_entry.native_relocation_ready, actual_entry.native_relocation_ready
            ));
        }
    }
    issues
}

fn push_object_section_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "object_section[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_object_section_string_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "object_section[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_object_relocation_seed_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "object_relocation_seed[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_object_relocation_seed_string_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "object_relocation_seed[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn toml_table_blocks<'a>(source: &'a str, table: &str) -> Vec<Vec<&'a str>> {
    let header = format!("[[{table}]]");
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut in_target_table = false;

    for raw in source.lines() {
        let line = raw.trim();
        if line.starts_with("[[") && line.ends_with("]]") {
            if in_target_table {
                blocks.push(current);
                current = Vec::new();
            }
            in_target_table = line == header;
            continue;
        }
        if in_target_table {
            current.push(line);
        }
    }
    if in_target_table {
        blocks.push(current);
    }

    blocks
}

fn toml_block_string_value(block: &[&str], key: &str) -> Option<String> {
    toml_block_value(block, key).and_then(toml_decode_string_value)
}

fn toml_block_usize_value(block: &[&str], key: &str) -> Option<usize> {
    toml_block_value(block, key).and_then(|value| value.parse::<usize>().ok())
}

fn toml_block_isize_value(block: &[&str], key: &str) -> Option<isize> {
    toml_block_value(block, key).and_then(|value| value.parse::<isize>().ok())
}

fn toml_block_bool_value(block: &[&str], key: &str) -> Option<bool> {
    toml_block_value(block, key).and_then(|value| value.parse::<bool>().ok())
}

fn toml_block_value<'a>(block: &'a [&'a str], key: &str) -> Option<&'a str> {
    block.iter().find_map(|line| {
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim())
    })
}

fn toml_decode_string_value(value: &str) -> Option<String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|value| {
            value
                .replace("\\n", "\n")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
        })
}

fn object_section_name(section_kind: &str, order_index: usize) -> String {
    match section_kind {
        "compiled-artifact" => ".nuis.text.compiled".to_owned(),
        "nsld-link-input-table" => ".nuis.meta.link_inputs".to_owned(),
        "nsld-link-unit-table" => ".nuis.meta.link_units".to_owned(),
        "nsld-link-bundle" => ".nuis.meta.link_bundle".to_owned(),
        "lowering-sidecar-input" => format!(".nuis.ir.sidecar.{order_index:04}"),
        "hetero-data-segment" => format!(".nuis.data.hetero.{order_index:04}"),
        other => format!(
            ".nuis.section.{}.{}",
            order_index,
            sanitize_section_token(other)
        ),
    }
}

fn object_section_layout(
    sections: &[super::reports::NsldAssembleSectionDiagnostic],
) -> Vec<NsldObjectSectionDiagnostic> {
    let mut next_file_offset_seed = 0usize;
    sections
        .iter()
        .map(|section| {
            let alignment = object_section_alignment(&section.section_kind);
            next_file_offset_seed = align_to(next_file_offset_seed, alignment);
            let source_size_bytes = source_size_bytes(&section.source_path);
            let file_offset_seed = next_file_offset_seed;
            let file_size_seed = source_size_bytes;
            next_file_offset_seed = next_file_offset_seed.saturating_add(file_size_seed);
            NsldObjectSectionDiagnostic {
                order_index: section.order_index,
                source_section_id: section.section_id.clone(),
                source_section_kind: section.section_kind.clone(),
                object_section_name: object_section_name(
                    &section.section_kind,
                    section.order_index,
                ),
                object_section_role: object_section_role(&section.section_kind),
                source_path: section.source_path.clone(),
                source_hash: section.source_hash.clone(),
                source_size_bytes,
                payload_offset_seed: section.order_index,
                file_offset_seed,
                file_size_seed,
                alignment,
                required: section.required,
            }
        })
        .collect()
}

fn object_relocation_seeds(
    object_sections: &[NsldObjectSectionDiagnostic],
) -> Vec<NsldObjectRelocationSeedDiagnostic> {
    object_sections
        .iter()
        .filter(|section| section.required)
        .enumerate()
        .map(|(index, section)| NsldObjectRelocationSeedDiagnostic {
            order_index: index,
            relocation_seed_id: format!(
                "orel{index:04}.{}",
                sanitize_section_token(&section.source_section_kind)
            ),
            relocation_seed_kind: object_relocation_seed_kind(&section.object_section_role),
            source_section_id: section.source_section_id.clone(),
            source_offset_seed: 0,
            target_symbol: format!(
                "__nuis_section_{}",
                sanitize_section_token(&section.source_section_id)
            ),
            addend: 0,
            native_relocation_ready: false,
        })
        .collect()
}

fn object_relocation_seed_kind(object_section_role: &str) -> String {
    match object_section_role {
        "native-bootstrap-input" => "bootstrap-entry-seed".to_owned(),
        "metadata" => "metadata-address-seed".to_owned(),
        "data" => "data-address-seed".to_owned(),
        _ => "extension-address-seed".to_owned(),
    }
}

fn object_section_alignment(section_kind: &str) -> usize {
    match section_kind {
        "compiled-artifact" => 16,
        "hetero-data-segment" => 16,
        _ => 8,
    }
}

fn align_to(value: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return value;
    }
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

fn source_size_bytes(source_path: &str) -> usize {
    fs::metadata(source_path)
        .map(|metadata| metadata.len() as usize)
        .unwrap_or(0)
}

fn object_section_role(section_kind: &str) -> String {
    match section_kind {
        "compiled-artifact" => "native-bootstrap-input".to_owned(),
        "nsld-link-input-table"
        | "nsld-link-unit-table"
        | "nsld-link-bundle"
        | "lowering-sidecar-input" => "metadata".to_owned(),
        "hetero-data-segment" => "data".to_owned(),
        _ => "extension".to_owned(),
    }
}

fn sanitize_section_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{nsld_object_plan_report, nsld_verify_object_plan_report};
    use crate::main_test_support::empty_link_plan;
    use std::{fs, path::Path};

    #[test]
    fn object_plan_is_plan_only_until_object_writer_exists() {
        let plan = empty_link_plan();
        let report = nsld_object_plan_report(Path::new("nuis.build.manifest.toml"), &plan);

        assert_eq!(report.object_format, "mach-o");
        assert_eq!(report.emission_status, "plan-only");
        assert_eq!(report.writer_target_id, "arm64-macos-mach-o");
        assert_eq!(report.writer_status, "blocked");
        assert_eq!(
            report.unsupported_features,
            vec![
                "object-byte-emitter".to_owned(),
                "native-relocation-applier".to_owned()
            ]
        );
        assert_eq!(
            report.object_sections[0].object_section_name,
            ".nuis.text.compiled"
        );
        assert_eq!(
            report.object_sections[0].object_section_role,
            "native-bootstrap-input"
        );
        assert_eq!(report.object_sections[0].alignment, 16);
        assert_eq!(report.object_sections[0].file_offset_seed, 0);
        assert_eq!(report.relocation_seed_count, report.relocation_seeds.len());
        assert!(report.object_layout_hash.starts_with("0x"));
        assert!(report.relocation_seed_table_hash.starts_with("0x"));
        assert_eq!(
            report.relocation_seeds[0].relocation_seed_kind,
            "bootstrap-entry-seed"
        );
        assert!(!report.relocation_seeds[0].native_relocation_ready);
        assert!(report
            .blockers
            .contains(&"object-byte-emitter:not-implemented".to_owned()));
        assert!(report
            .blockers
            .contains(&"native-relocation-applier:not-implemented".to_owned()));
    }

    #[test]
    fn verify_object_plan_reports_missing_object_section_fields() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-field-tamper-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report)
            .replace("object_section_role = \"", "# object_section_role = \"");
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "object_section[0].object_section_role missing"));
    }

    #[test]
    fn verify_object_plan_reports_object_section_name_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-section-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report)
            .replace(".nuis.text.compiled", ".nuis.text.wrong");
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue
                == "object_section[0].object_section_name mismatch: expected .nuis.text.compiled, found .nuis.text.wrong"
        }));
    }

    #[test]
    fn verify_object_plan_reports_relocation_seed_drift() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-relocation-seed-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report).replace(
            "relocation_seed_kind = \"bootstrap-entry-seed\"",
            "relocation_seed_kind = \"wrong-seed\"",
        );
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue
                == "object_relocation_seed[0].relocation_seed_kind mismatch: expected bootstrap-entry-seed, found wrong-seed"
        }));
    }

    #[test]
    fn verify_object_plan_reports_missing_writer_header_fields() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-writer-header-tamper-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report)
            .replace("writer_status = \"", "# writer_status = \"");
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "object_plan_header[0].writer_status missing"));
    }

    #[test]
    fn object_writer_readiness_stays_blocked_until_writer_exists() {
        let plan = empty_link_plan();
        let report = super::nsld_object_writer_readiness_report(Path::new("manifest.toml"), &plan);

        assert!(!report.can_emit_object);
        assert_eq!(report.writer_status, "blocked");
        assert!(report
            .unsupported_features
            .contains(&"object-byte-emitter".to_owned()));
    }

    #[test]
    fn emit_object_reports_blocked_state_without_writing_bytes() {
        let dir =
            std::env::temp_dir().join(format!("nsld-object-emit-blocked-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        let report = super::nsld_emit_object_report(Path::new("manifest.toml"), &plan).unwrap();
        let blocked_report = fs::read_to_string(dir.join("nuis.nsld.object.blocked.toml")).unwrap();
        fs::remove_dir_all(dir).unwrap();

        assert!(!report.emitted);
        assert!(!report.can_emit_object);
        assert!(report.output_path.ends_with("nuis.nsld.mach-o"));
        assert!(report
            .blocked_report_path
            .ends_with("nuis.nsld.object.blocked.toml"));
        assert!(blocked_report.contains("kind = \"object-emit-blocked\""));
        assert!(blocked_report.contains("emitted = false"));
        assert!(report
            .blockers
            .contains(&"object-byte-emitter:not-implemented".to_owned()));
    }
}
