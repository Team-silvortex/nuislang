use super::{
    fnv1a64_hex,
    object_file_layout::nsld_object_file_layout_report,
    object_image_backend::{
        encode_object_image_for_backend, object_image_backend_capabilities,
        object_image_backend_family, object_image_backend_relocation_lowering_rule_count,
        object_image_backend_relocation_lowering_rules, object_image_backend_relocation_records,
        object_image_backend_status,
    },
    object_plan_verify::{
        toml_block_bool_value, toml_block_string_value, toml_block_usize_value, toml_table_blocks,
    },
    reports::{
        NsldObjectImageDryRunEmitReport, NsldObjectImageDryRunReport,
        NsldObjectImageDryRunVerifyReport, NsldObjectImageRelocationRecordDiagnostic,
        NsldRelocationLoweringRuleDiagnostic,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(test)]
#[path = "object_image_dry_run_tests.rs"]
mod tests;

pub(crate) fn nsld_object_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectImageDryRunReport {
    let file_layout = nsld_object_file_layout_report(manifest, plan);
    let image_result = encode_object_image_for_backend(manifest, plan, &file_layout);
    let mut blockers = file_layout.blockers.clone();
    if !file_layout.layout_ready {
        blockers.push("object-file-layout:not-ready".to_owned());
    }
    blockers.extend(image_result.blockers);
    let relocation_lowering_issues = blockers
        .iter()
        .filter(|blocker| blocker.starts_with("mach-o-relocation:"))
        .cloned()
        .collect::<Vec<_>>();
    let relocation_lowering_valid = relocation_lowering_issues.is_empty();
    let relocation_lowering_rule_count =
        object_image_backend_relocation_lowering_rule_count(&file_layout.writer_backend_kind);
    let relocation_lowering_rules =
        object_image_backend_relocation_lowering_rules(&file_layout.writer_backend_kind);
    let relocation_records = object_image_backend_relocation_records(
        &file_layout.writer_backend_kind,
        manifest,
        plan,
        &file_layout,
    );
    let relocation_record_count = relocation_records.len();
    let relocation_record_table_hash = relocation_record_table_hash(&relocation_records);
    let image = image_result.image;
    let image_size_bytes = image.as_ref().map(Vec::len);
    let image_hash = image.as_ref().map(|bytes| fnv1a64_hex(bytes));
    let image_constructed = image.is_some();
    let image_ready = image_constructed && file_layout.layout_ready && blockers.is_empty();

    NsldObjectImageDryRunReport {
        manifest: manifest.display().to_string(),
        output_path: PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.object-image-dry-run.toml")
            .display()
            .to_string(),
        image_path: object_image_dry_run_image_path(plan).display().to_string(),
        writer_target_id: file_layout.writer_target_id,
        writer_backend_kind: file_layout.writer_backend_kind.clone(),
        object_family: file_layout.object_family,
        backend_family: object_image_backend_family(&file_layout.writer_backend_kind).to_owned(),
        backend_status: object_image_backend_status(&file_layout.writer_backend_kind).to_owned(),
        backend_capabilities: object_image_backend_capabilities(&file_layout.writer_backend_kind),
        backend_kind: file_layout.writer_backend_kind,
        object_format: file_layout.object_format,
        file_layout_hash: file_layout.file_layout_hash,
        record_count: file_layout.record_count,
        total_file_size_bytes: file_layout.total_file_size_bytes,
        image_constructed,
        image_ready,
        image_size_bytes,
        image_hash,
        relocation_lowering_valid,
        relocation_lowering_rule_count,
        relocation_lowering_rules,
        relocation_lowering_issues,
        relocation_record_count,
        relocation_record_table_hash,
        relocation_records,
        blockers,
    }
}

pub(crate) fn nsld_emit_object_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectImageDryRunEmitReport, String> {
    let report = nsld_object_image_dry_run_report(manifest, plan);
    let image = encode_object_image_dry_run(manifest, plan);
    let image_emitted = match image {
        Some(bytes) => {
            fs::write(&report.image_path, bytes).map_err(|error| {
                format!(
                    "failed to write nsld object image dry run bytes `{}`: {error}",
                    report.image_path
                )
            })?;
            true
        }
        None => false,
    };
    fs::write(
        &report.output_path,
        toml::render_object_image_dry_run(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object image dry run `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldObjectImageDryRunEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        image_path: report.image_path,
        image_emitted,
        image_constructed: report.image_constructed,
        image_ready: report.image_ready,
        image_size_bytes: report.image_size_bytes,
        image_hash: report.image_hash,
    })
}

pub(crate) fn nsld_verify_object_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectImageDryRunVerifyReport {
    let expected_report = nsld_object_image_dry_run_report(manifest, plan);
    let expected = toml::render_object_image_dry_run(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-image-dry-run.toml");
    let image_path = object_image_dry_run_image_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_image_dry_run `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_file_layout_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_backend_family,
        actual_backend_status,
        actual_image_constructed,
        actual_image_ready,
        actual_image_size_bytes,
        actual_image_hash,
        actual_relocation_lowering_valid,
        actual_relocation_lowering_rule_count,
        actual_relocation_lowering_rules,
        actual_relocation_lowering_issues,
        actual_relocation_record_count,
        actual_relocation_record_table_hash,
        actual_relocation_records,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "file_layout_hash"),
            toml::string_value(source, "writer_backend_kind"),
            toml::string_value(source, "object_family"),
            toml::string_value(source, "backend_family"),
            toml::string_value(source, "backend_status"),
            toml::bool_value(source, "image_constructed"),
            toml::bool_value(source, "image_ready"),
            optional_usize_value(source, "image_size_bytes"),
            optional_string_value(source, "image_hash"),
            toml::bool_value(source, "relocation_lowering_valid"),
            toml::usize_value(source, "relocation_lowering_rule_count"),
            Some(relocation_lowering_rule_entries(source)),
            Some(toml::string_array_value(
                source,
                "relocation_lowering_issues",
            )),
            toml::usize_value(source, "relocation_record_count"),
            toml::string_value(source, "relocation_record_table_hash"),
            Some(relocation_record_entries(source)),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            )
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-image-dry-run-content-mismatch".to_owned());
        }
        push_string_mismatch(
            &mut issues,
            "file_layout_hash",
            &expected_report.file_layout_hash,
            actual_file_layout_hash.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "writer_backend_kind",
            &expected_report.writer_backend_kind,
            actual_writer_backend_kind.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "object_family",
            &expected_report.object_family,
            actual_object_family.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "backend_family",
            &expected_report.backend_family,
            actual_backend_family.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "backend_status",
            &expected_report.backend_status,
            actual_backend_status.as_deref(),
        );
        push_bool_mismatch(
            &mut issues,
            "image_constructed",
            expected_report.image_constructed,
            actual_image_constructed,
        );
        push_bool_mismatch(
            &mut issues,
            "image_ready",
            expected_report.image_ready,
            actual_image_ready,
        );
        push_optional_usize_mismatch(
            &mut issues,
            "image_size_bytes",
            expected_report.image_size_bytes,
            actual_image_size_bytes,
        );
        push_optional_string_mismatch(
            &mut issues,
            "image_hash",
            expected_report.image_hash.as_deref(),
            actual_image_hash.as_deref(),
        );
        push_bool_mismatch(
            &mut issues,
            "relocation_lowering_valid",
            expected_report.relocation_lowering_valid,
            actual_relocation_lowering_valid,
        );
        push_usize_mismatch(
            &mut issues,
            "relocation_lowering_rule_count",
            expected_report.relocation_lowering_rule_count,
            actual_relocation_lowering_rule_count,
        );
        push_string_array_mismatch(
            &mut issues,
            "relocation_lowering_issues",
            &expected_report.relocation_lowering_issues,
            actual_relocation_lowering_issues.as_deref(),
        );
        push_relocation_lowering_rule_mismatches(
            &mut issues,
            &expected_report.relocation_lowering_rules,
            actual_relocation_lowering_rules.as_deref(),
        );
        push_usize_mismatch(
            &mut issues,
            "relocation_record_count",
            expected_report.relocation_record_count,
            actual_relocation_record_count,
        );
        push_string_mismatch(
            &mut issues,
            "relocation_record_table_hash",
            &expected_report.relocation_record_table_hash,
            actual_relocation_record_table_hash.as_deref(),
        );
        push_relocation_record_mismatches(
            &mut issues,
            &expected_report.relocation_records,
            actual_relocation_records.as_deref(),
        );
    }
    let (actual_image_file_size_bytes, actual_image_file_hash) =
        image_file_size_and_hash(&image_path).unwrap_or_else(|error| {
            if expected_report.image_constructed {
                issues.push(error);
            }
            (None, None)
        });
    push_optional_usize_mismatch(
        &mut issues,
        "image_file_size_bytes",
        expected_report.image_size_bytes,
        actual_image_file_size_bytes,
    );
    push_optional_string_mismatch(
        &mut issues,
        "image_file_hash",
        expected_report.image_hash.as_deref(),
        actual_image_file_hash.as_deref(),
    );

    NsldObjectImageDryRunVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        image_path: image_path.display().to_string(),
        valid: issues.is_empty(),
        expected_writer_backend_kind: expected_report.writer_backend_kind,
        expected_object_family: expected_report.object_family,
        expected_backend_family: expected_report.backend_family,
        expected_backend_status: expected_report.backend_status,
        expected_file_layout_hash: expected_report.file_layout_hash,
        expected_image_constructed: expected_report.image_constructed,
        expected_image_ready: expected_report.image_ready,
        expected_image_size_bytes: expected_report.image_size_bytes,
        expected_image_hash: expected_report.image_hash,
        expected_relocation_lowering_valid: expected_report.relocation_lowering_valid,
        expected_relocation_lowering_rule_count: expected_report.relocation_lowering_rule_count,
        expected_relocation_lowering_rules: expected_report.relocation_lowering_rules,
        expected_relocation_lowering_issues: expected_report.relocation_lowering_issues,
        expected_relocation_record_count: expected_report.relocation_record_count,
        expected_relocation_record_table_hash: expected_report.relocation_record_table_hash,
        expected_relocation_records: expected_report.relocation_records,
        actual_file_layout_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_backend_family,
        actual_backend_status,
        actual_image_constructed,
        actual_image_ready,
        actual_image_size_bytes,
        actual_image_hash,
        actual_relocation_lowering_valid,
        actual_relocation_lowering_rule_count,
        actual_relocation_lowering_rules,
        actual_relocation_lowering_issues,
        actual_relocation_record_count,
        actual_relocation_record_table_hash,
        actual_relocation_records,
        actual_image_file_size_bytes,
        actual_image_file_hash,
        issues,
    }
}

fn encode_object_image_dry_run(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> Option<Vec<u8>> {
    let file_layout = nsld_object_file_layout_report(manifest, plan);
    encode_object_image_for_backend(manifest, plan, &file_layout).image
}

fn object_image_dry_run_image_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    PathBuf::from(&plan.output_dir).join("nuis.nsld.object-image-dry-run.bin")
}

fn image_file_size_and_hash(path: &Path) -> Result<(Option<usize>, Option<String>), String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_image_dry_run_bytes `{}`: {error}",
            path.display()
        )
    })?;
    Ok((Some(bytes.len()), Some(fnv1a64_hex(&bytes))))
}

fn optional_string_value(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn optional_usize_value(source: &str, key: &str) -> Option<usize> {
    toml::usize_value(source, key).filter(|value| *value != 0)
}

fn relocation_lowering_rule_entries(source: &str) -> Vec<NsldRelocationLoweringRuleDiagnostic> {
    toml_table_blocks(source, "relocation_lowering_rule")
        .into_iter()
        .filter_map(|block| {
            Some(NsldRelocationLoweringRuleDiagnostic {
                rule_id: toml_block_string_value(&block, "rule_id")?,
                source_seed_kind: toml_block_string_value(&block, "source_seed_kind")?,
                target_relocation_kind: toml_block_string_value(&block, "target_relocation_kind")?,
                pc_relative: toml_block_bool_value(&block, "pc_relative")?,
                length_power: toml_block_usize_value(&block, "length_power")? as u8,
                external: toml_block_bool_value(&block, "external")?,
                relocation_type: toml_block_usize_value(&block, "relocation_type")? as u8,
            })
        })
        .collect()
}

fn relocation_record_entries(source: &str) -> Vec<NsldObjectImageRelocationRecordDiagnostic> {
    toml_table_blocks(source, "relocation_record")
        .into_iter()
        .filter_map(|block| {
            Some(NsldObjectImageRelocationRecordDiagnostic {
                record_id: toml_block_string_value(&block, "record_id")?,
                relocation_seed_id: toml_block_string_value(&block, "relocation_seed_id")?,
                source_section_id: toml_block_string_value(&block, "source_section_id")?,
                source_offset: toml_block_usize_value(&block, "source_offset")?,
                source_seed_kind: toml_block_string_value(&block, "source_seed_kind")?,
                target_relocation_kind: toml_block_string_value(&block, "target_relocation_kind")?,
                symbol_index: toml_block_usize_value(&block, "symbol_index")? as u32,
                pc_relative: toml_block_bool_value(&block, "pc_relative")?,
                length_power: toml_block_usize_value(&block, "length_power")? as u8,
                external: toml_block_bool_value(&block, "external")?,
                relocation_type: toml_block_usize_value(&block, "relocation_type")? as u8,
            })
        })
        .collect()
}

fn relocation_record_table_hash(records: &[NsldObjectImageRelocationRecordDiagnostic]) -> String {
    let mut material = String::new();
    for record in records {
        material.push_str(&record.record_id);
        material.push('\t');
        material.push_str(&record.relocation_seed_id);
        material.push('\t');
        material.push_str(&record.source_section_id);
        material.push('\t');
        material.push_str(&record.source_offset.to_string());
        material.push('\t');
        material.push_str(&record.source_seed_kind);
        material.push('\t');
        material.push_str(&record.target_relocation_kind);
        material.push('\t');
        material.push_str(&record.symbol_index.to_string());
        material.push('\t');
        material.push_str(&record.pc_relative.to_string());
        material.push('\t');
        material.push_str(&record.length_power.to_string());
        material.push('\t');
        material.push_str(&record.external.to_string());
        material.push('\t');
        material.push_str(&record.relocation_type.to_string());
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn push_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &str,
    actual: Option<&str>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual.unwrap_or("missing")
        ));
    }
}

fn push_bool_mismatch(issues: &mut Vec<String>, field: &str, expected: bool, actual: Option<bool>) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn push_usize_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: usize,
    actual: Option<usize>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn push_optional_usize_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: Option<usize>,
    actual: Option<usize>,
) {
    if actual != expected {
        issues.push(format!(
            "{field} mismatch: expected {}, found {}",
            optional_usize_text(expected),
            optional_usize_text(actual)
        ));
    }
}

fn push_string_array_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &[String],
    actual: Option<&[String]>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected [{}], found [{}]",
            expected.join(", "),
            actual
                .map(|values| values.join(", "))
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn push_relocation_lowering_rule_mismatches(
    issues: &mut Vec<String>,
    expected: &[NsldRelocationLoweringRuleDiagnostic],
    actual: Option<&[NsldRelocationLoweringRuleDiagnostic]>,
) {
    let Some(actual) = actual else {
        issues.push(format!(
            "relocation_lowering_rule_entry_count mismatch: expected {}, found missing",
            expected.len()
        ));
        return;
    };
    if actual.len() != expected.len() {
        issues.push(format!(
            "relocation_lowering_rule_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_rule) in expected.iter().enumerate() {
        let Some(actual_rule) = actual.get(index) else {
            issues.push(format!("relocation_lowering_rule[{index}] missing"));
            continue;
        };
        push_rule_string_mismatch(
            issues,
            index,
            "rule_id",
            &expected_rule.rule_id,
            &actual_rule.rule_id,
        );
        push_rule_string_mismatch(
            issues,
            index,
            "source_seed_kind",
            &expected_rule.source_seed_kind,
            &actual_rule.source_seed_kind,
        );
        push_rule_string_mismatch(
            issues,
            index,
            "target_relocation_kind",
            &expected_rule.target_relocation_kind,
            &actual_rule.target_relocation_kind,
        );
        push_rule_bool_mismatch(
            issues,
            index,
            "pc_relative",
            expected_rule.pc_relative,
            actual_rule.pc_relative,
        );
        push_rule_usize_mismatch(
            issues,
            index,
            "length_power",
            expected_rule.length_power as usize,
            actual_rule.length_power as usize,
        );
        push_rule_bool_mismatch(
            issues,
            index,
            "external",
            expected_rule.external,
            actual_rule.external,
        );
        push_rule_usize_mismatch(
            issues,
            index,
            "relocation_type",
            expected_rule.relocation_type as usize,
            actual_rule.relocation_type as usize,
        );
    }
}

fn push_relocation_record_mismatches(
    issues: &mut Vec<String>,
    expected: &[NsldObjectImageRelocationRecordDiagnostic],
    actual: Option<&[NsldObjectImageRelocationRecordDiagnostic]>,
) {
    let Some(actual) = actual else {
        issues.push(format!(
            "relocation_record_entry_count mismatch: expected {}, found missing",
            expected.len()
        ));
        return;
    };
    if actual.len() != expected.len() {
        issues.push(format!(
            "relocation_record_entry_count mismatch: expected {}, found {}",
            expected.len(),
            actual.len()
        ));
    }
    for (index, expected_record) in expected.iter().enumerate() {
        let Some(actual_record) = actual.get(index) else {
            issues.push(format!("relocation_record[{index}] missing"));
            continue;
        };
        push_record_string_mismatch(
            issues,
            index,
            "record_id",
            &expected_record.record_id,
            &actual_record.record_id,
        );
        push_record_string_mismatch(
            issues,
            index,
            "relocation_seed_id",
            &expected_record.relocation_seed_id,
            &actual_record.relocation_seed_id,
        );
        push_record_string_mismatch(
            issues,
            index,
            "source_section_id",
            &expected_record.source_section_id,
            &actual_record.source_section_id,
        );
        push_record_usize_mismatch(
            issues,
            index,
            "source_offset",
            expected_record.source_offset,
            actual_record.source_offset,
        );
        push_record_string_mismatch(
            issues,
            index,
            "source_seed_kind",
            &expected_record.source_seed_kind,
            &actual_record.source_seed_kind,
        );
        push_record_string_mismatch(
            issues,
            index,
            "target_relocation_kind",
            &expected_record.target_relocation_kind,
            &actual_record.target_relocation_kind,
        );
        push_record_usize_mismatch(
            issues,
            index,
            "symbol_index",
            expected_record.symbol_index as usize,
            actual_record.symbol_index as usize,
        );
        push_record_bool_mismatch(
            issues,
            index,
            "pc_relative",
            expected_record.pc_relative,
            actual_record.pc_relative,
        );
        push_record_usize_mismatch(
            issues,
            index,
            "length_power",
            expected_record.length_power as usize,
            actual_record.length_power as usize,
        );
        push_record_bool_mismatch(
            issues,
            index,
            "external",
            expected_record.external,
            actual_record.external,
        );
        push_record_usize_mismatch(
            issues,
            index,
            "relocation_type",
            expected_record.relocation_type as usize,
            actual_record.relocation_type as usize,
        );
    }
}

fn push_record_string_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "relocation_record[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_record_bool_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: bool,
    actual: bool,
) {
    if actual != expected {
        issues.push(format!(
            "relocation_record[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_record_usize_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "relocation_record[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_rule_string_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: &str,
    actual: &str,
) {
    if actual != expected {
        issues.push(format!(
            "relocation_lowering_rule[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_rule_bool_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: bool,
    actual: bool,
) {
    if actual != expected {
        issues.push(format!(
            "relocation_lowering_rule[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_rule_usize_mismatch(
    issues: &mut Vec<String>,
    index: usize,
    field: &str,
    expected: usize,
    actual: usize,
) {
    if actual != expected {
        issues.push(format!(
            "relocation_lowering_rule[{index}].{field} mismatch: expected {expected}, found {actual}"
        ));
    }
}

fn push_optional_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if actual != expected {
        issues.push(format!(
            "{field} mismatch: expected {}, found {}",
            expected.unwrap_or("missing"),
            actual.unwrap_or("missing")
        ));
    }
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}
