use super::{
    fnv1a64_hex,
    object_plan_verify::{
        toml_block_bool_value, toml_block_string_value, toml_block_usize_value, toml_table_blocks,
    },
    reports::{NsldObjectImageRelocationRecordDiagnostic, NsldRelocationLoweringRuleDiagnostic},
    toml,
};

pub(crate) fn optional_string_value(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

pub(crate) fn optional_usize_value(source: &str, key: &str) -> Option<usize> {
    toml::usize_value(source, key).filter(|value| *value != 0)
}

pub(crate) fn relocation_lowering_rule_entries(
    source: &str,
) -> Vec<NsldRelocationLoweringRuleDiagnostic> {
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

pub(crate) fn relocation_record_entries(
    source: &str,
) -> Vec<NsldObjectImageRelocationRecordDiagnostic> {
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

pub(crate) fn relocation_record_table_hash(
    records: &[NsldObjectImageRelocationRecordDiagnostic],
) -> String {
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

pub(crate) fn push_string_mismatch(
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

pub(crate) fn push_bool_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: bool,
    actual: Option<bool>,
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

pub(crate) fn push_usize_mismatch(
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

pub(crate) fn push_optional_usize_mismatch(
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

pub(crate) fn push_string_array_mismatch(
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

pub(crate) fn push_relocation_lowering_rule_mismatches(
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

pub(crate) fn push_relocation_record_mismatches(
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

pub(crate) fn push_optional_string_mismatch(
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
