use std::{collections::BTreeMap, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError,
};

pub const COMPILER_DIAGNOSTIC_REPORT_PROTOCOL: &str = "nuis-compiler-diagnostic-report-v1";
pub const COMPILER_DIAGNOSTIC_NORMALIZATION_CONTRACT: &str =
    "nuis-compiler-diagnostic-normalization-v1";
pub const COMPILER_DIAGNOSTIC_REPORT_FILE: &str = "nuis.compiler-diagnostics.toml";
const DIAGNOSTIC_RECORD_IDENTITY_CONTRACT: &str = "nuis-compiler-diagnostic-record-v1";

#[derive(Debug, Clone, Copy)]
pub struct CompilerDiagnosticInput<'a> {
    pub module: &'a str,
    pub code: &'a str,
    pub path: &'a str,
    pub message: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerDiagnosticReportInput<'a> {
    pub producer_id: &'a str,
    pub component_record_sha256: &'a str,
    pub bootstrap_subset_protocol: &'a str,
    pub accepted: bool,
    pub semantic_pipeline: &'a str,
    pub semantic_error: Option<&'a str>,
    pub diagnostics: &'a [CompilerDiagnosticInput<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerDiagnosticRecord {
    pub ordinal: usize,
    pub module: String,
    pub code: String,
    pub path: String,
    pub message: String,
    pub record_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerDiagnosticReport {
    pub protocol: String,
    pub normalization_contract: String,
    pub producer_id: String,
    pub component_record_sha256: String,
    pub bootstrap_subset_protocol: String,
    pub accepted: bool,
    pub semantic_pipeline: String,
    pub semantic_error: String,
    pub diagnostic_count: usize,
    pub diagnostics_sha256: String,
    pub report_sha256: String,
    pub diagnostics: Vec<CompilerDiagnosticRecord>,
}

pub fn build_compiler_diagnostic_report(
    input: &CompilerDiagnosticReportInput<'_>,
) -> Result<CompilerDiagnosticReport, ArtifactError> {
    validate_token(input.producer_id, "producer id")?;
    validate_token(input.bootstrap_subset_protocol, "bootstrap subset protocol")?;
    validate_sha256(input.component_record_sha256, "component record")?;
    validate_pipeline(input.semantic_pipeline)?;
    let semantic_error = input.semantic_error.unwrap_or_default();
    validate_diagnostic_text(semantic_error, "semantic error", true)?;

    let mut ordered = input.diagnostics.to_vec();
    ordered.sort_by(|lhs, rhs| {
        (lhs.module, lhs.code, lhs.path, lhs.message).cmp(&(
            rhs.module,
            rhs.code,
            rhs.path,
            rhs.message,
        ))
    });
    let mut diagnostics = Vec::with_capacity(ordered.len());
    for (ordinal, diagnostic) in ordered.iter().enumerate() {
        validate_canonical_string(diagnostic.module, "diagnostic module")?;
        validate_token(diagnostic.code, "diagnostic code")?;
        validate_diagnostic_path(diagnostic.path)?;
        validate_diagnostic_text(diagnostic.message, "diagnostic message", false)?;
        diagnostics.push(CompilerDiagnosticRecord {
            ordinal,
            module: diagnostic.module.to_owned(),
            code: diagnostic.code.to_owned(),
            path: diagnostic.path.to_owned(),
            message: diagnostic.message.to_owned(),
            record_sha256: diagnostic_record_identity(ordinal, diagnostic),
        });
    }

    let diagnostics_sha256 = diagnostics_identity(
        input.accepted,
        input.semantic_pipeline,
        semantic_error,
        &diagnostics,
    );
    let mut report = CompilerDiagnosticReport {
        protocol: COMPILER_DIAGNOSTIC_REPORT_PROTOCOL.to_owned(),
        normalization_contract: COMPILER_DIAGNOSTIC_NORMALIZATION_CONTRACT.to_owned(),
        producer_id: input.producer_id.to_owned(),
        component_record_sha256: input.component_record_sha256.to_owned(),
        bootstrap_subset_protocol: input.bootstrap_subset_protocol.to_owned(),
        accepted: input.accepted,
        semantic_pipeline: input.semantic_pipeline.to_owned(),
        semantic_error: semantic_error.to_owned(),
        diagnostic_count: diagnostics.len(),
        diagnostics_sha256,
        report_sha256: String::new(),
        diagnostics,
    };
    report.report_sha256 = report_identity(&report);
    validate_compiler_diagnostic_report(&report)?;
    Ok(report)
}

pub fn render_compiler_diagnostic_report(report: &CompilerDiagnosticReport) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nnormalization_contract = \"{}\"\nproducer_id = \"{}\"\ncomponent_record_sha256 = \"{}\"\nbootstrap_subset_protocol = \"{}\"\naccepted = {}\nsemantic_pipeline = \"{}\"\nsemantic_error = \"{}\"\ndiagnostic_count = {}\ndiagnostics_sha256 = \"{}\"\nreport_sha256 = \"{}\"\n",
        report.protocol,
        report.normalization_contract,
        escape_toml_string(&report.producer_id),
        report.component_record_sha256,
        escape_toml_string(&report.bootstrap_subset_protocol),
        report.accepted,
        escape_toml_string(&report.semantic_pipeline),
        escape_toml_string(&report.semantic_error),
        report.diagnostic_count,
        report.diagnostics_sha256,
        report.report_sha256,
    );
    for diagnostic in &report.diagnostics {
        out.push_str(&format!(
            "\n[[diagnostic]]\nordinal = {}\nmodule = \"{}\"\ncode = \"{}\"\npath = \"{}\"\nmessage = \"{}\"\nrecord_sha256 = \"{}\"\n",
            diagnostic.ordinal,
            escape_toml_string(&diagnostic.module),
            escape_toml_string(&diagnostic.code),
            escape_toml_string(&diagnostic.path),
            escape_toml_string(&diagnostic.message),
            diagnostic.record_sha256,
        ));
    }
    out
}

pub fn parse_compiler_diagnostic_report(
    path: &Path,
) -> Result<CompilerDiagnosticReport, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler diagnostic report `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_diagnostic_report_from_source(&source, path)
}

pub fn parse_compiler_diagnostic_report_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerDiagnosticReport, ArtifactError> {
    validate_text(source, path)?;
    let report = CompilerDiagnosticReport {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        normalization_contract: parse_required_toml_string(source, "normalization_contract", path)?,
        producer_id: parse_required_toml_string(source, "producer_id", path)?,
        component_record_sha256: parse_required_toml_string(
            source,
            "component_record_sha256",
            path,
        )?,
        bootstrap_subset_protocol: parse_required_toml_string(
            source,
            "bootstrap_subset_protocol",
            path,
        )?,
        accepted: parse_required_toml_bool(source, "accepted", path)?,
        semantic_pipeline: parse_required_toml_string(source, "semantic_pipeline", path)?,
        semantic_error: parse_required_toml_string(source, "semantic_error", path)?,
        diagnostic_count: parse_required_toml_usize(source, "diagnostic_count", path)?,
        diagnostics_sha256: parse_required_toml_string(source, "diagnostics_sha256", path)?,
        report_sha256: parse_required_toml_string(source, "report_sha256", path)?,
        diagnostics: parse_diagnostic_blocks(source, path)?,
    };
    validate_compiler_diagnostic_report(&report)?;
    if render_compiler_diagnostic_report(&report) != source {
        return Err(ArtifactError::new(format!(
            "compiler diagnostic report `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(report)
}

pub fn read_compiler_diagnostic_report(
    path: &Path,
    component_record_sha256: &str,
    producer_id: &str,
) -> Result<CompilerDiagnosticReport, ArtifactError> {
    let report = parse_compiler_diagnostic_report(path)?;
    if report.component_record_sha256 != component_record_sha256
        || report.producer_id != producer_id
    {
        return Err(ArtifactError::new(
            "compiler diagnostic report does not match its component record",
        ));
    }
    Ok(report)
}

fn parse_diagnostic_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerDiagnosticRecord>, ArtifactError> {
    let mut records = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[diagnostic]]" {
            if in_block {
                records.push(parse_diagnostic(&values, path)?);
                values.clear();
            }
            in_block = true;
            continue;
        }
        if in_block && !line.is_empty() && !line.starts_with('#') {
            let Some((key, value)) = line.split_once('=') else {
                return Err(ArtifactError::new(format!(
                    "`{}` contains malformed compiler diagnostic line `{line}`",
                    path.display()
                )));
            };
            let key = key.trim().to_owned();
            if values
                .insert(key.clone(), value.trim().to_owned())
                .is_some()
            {
                return Err(ArtifactError::new(format!(
                    "`{}` compiler diagnostic repeats key `{key}`",
                    path.display()
                )));
            }
        }
    }
    if in_block {
        records.push(parse_diagnostic(&values, path)?);
    }
    Ok(records)
}

fn parse_diagnostic(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerDiagnosticRecord, ArtifactError> {
    Ok(CompilerDiagnosticRecord {
        ordinal: parse_optional_map_usize(values, "ordinal", path, "diagnostic")?.ok_or_else(
            || {
                ArtifactError::new(format!(
                    "`{}` diagnostic is missing `ordinal`",
                    path.display()
                ))
            },
        )?,
        module: parse_required_map_string_in_block(values, "module", path, "diagnostic")?,
        code: parse_required_map_string_in_block(values, "code", path, "diagnostic")?,
        path: parse_required_map_string_in_block(values, "path", path, "diagnostic")?,
        message: parse_required_map_string_in_block(values, "message", path, "diagnostic")?,
        record_sha256: parse_required_map_string_in_block(
            values,
            "record_sha256",
            path,
            "diagnostic",
        )?,
    })
}

fn validate_compiler_diagnostic_report(
    report: &CompilerDiagnosticReport,
) -> Result<(), ArtifactError> {
    if report.protocol != COMPILER_DIAGNOSTIC_REPORT_PROTOCOL
        || report.normalization_contract != COMPILER_DIAGNOSTIC_NORMALIZATION_CONTRACT
    {
        return Err(ArtifactError::new(
            "compiler diagnostic report declares an unsupported protocol contract",
        ));
    }
    validate_token(&report.producer_id, "producer id")?;
    validate_token(
        &report.bootstrap_subset_protocol,
        "bootstrap subset protocol",
    )?;
    validate_sha256(&report.component_record_sha256, "component record")?;
    validate_pipeline(&report.semantic_pipeline)?;
    validate_diagnostic_text(&report.semantic_error, "semantic error", true)?;
    if report.accepted
        && (report.semantic_pipeline != "checked"
            || !report.semantic_error.is_empty()
            || !report.diagnostics.is_empty())
    {
        return Err(ArtifactError::new(
            "accepted compiler diagnostic reports must be checked and diagnostic-free",
        ));
    }
    if report.diagnostic_count != report.diagnostics.len() {
        return Err(ArtifactError::new(
            "compiler diagnostic report count does not match its records",
        ));
    }
    let mut previous = None;
    for (ordinal, diagnostic) in report.diagnostics.iter().enumerate() {
        if diagnostic.ordinal != ordinal {
            return Err(ArtifactError::new(
                "compiler diagnostic records must use canonical ordinals",
            ));
        }
        validate_canonical_string(&diagnostic.module, "diagnostic module")?;
        validate_token(&diagnostic.code, "diagnostic code")?;
        validate_diagnostic_path(&diagnostic.path)?;
        validate_diagnostic_text(&diagnostic.message, "diagnostic message", false)?;
        let key = (
            diagnostic.module.as_str(),
            diagnostic.code.as_str(),
            diagnostic.path.as_str(),
            diagnostic.message.as_str(),
        );
        if previous.is_some_and(|previous| previous > key) {
            return Err(ArtifactError::new(
                "compiler diagnostic records are not canonically sorted",
            ));
        }
        previous = Some(key);
        let input = CompilerDiagnosticInput {
            module: &diagnostic.module,
            code: &diagnostic.code,
            path: &diagnostic.path,
            message: &diagnostic.message,
        };
        if diagnostic.record_sha256 != diagnostic_record_identity(ordinal, &input) {
            return Err(ArtifactError::new(
                "compiler diagnostic record identity mismatch",
            ));
        }
    }
    let diagnostics_sha256 = diagnostics_identity(
        report.accepted,
        &report.semantic_pipeline,
        &report.semantic_error,
        &report.diagnostics,
    );
    if report.diagnostics_sha256 != diagnostics_sha256 {
        return Err(ArtifactError::new(
            "compiler diagnostic normalization identity mismatch",
        ));
    }
    validate_sha256(&report.diagnostics_sha256, "normalized diagnostics")?;
    validate_sha256(&report.report_sha256, "diagnostic report")?;
    if report.report_sha256 != report_identity(report) {
        return Err(ArtifactError::new(
            "compiler diagnostic report identity mismatch",
        ));
    }
    Ok(())
}

fn validate_token(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.is_empty()
        || value.len() > 256
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        })
    {
        return Err(ArtifactError::new(format!(
            "compiler diagnostic {label} contains unsupported characters"
        )));
    }
    Ok(())
}

fn validate_canonical_string(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(ArtifactError::new(format!(
            "compiler diagnostic {label} must be a non-empty canonical UTF-8 string"
        )));
    }
    Ok(())
}

fn validate_diagnostic_path(value: &str) -> Result<(), ArtifactError> {
    validate_canonical_string(value, "diagnostic path")?;
    let bytes = value.as_bytes();
    let has_drive_prefix = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if value.starts_with('/')
        || value.starts_with('\\')
        || value.contains('\\')
        || has_drive_prefix
        || value
            .split('/')
            .any(|component| matches!(component, "." | ".."))
    {
        return Err(ArtifactError::new(
            "compiler diagnostic path must be a portable logical path",
        ));
    }
    Ok(())
}

fn validate_pipeline(value: &str) -> Result<(), ArtifactError> {
    if matches!(value, "checked" | "failed" | "skipped") {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "unsupported compiler diagnostic semantic pipeline `{value}`"
        )))
    }
}

fn validate_diagnostic_text(
    value: &str,
    label: &str,
    allow_empty: bool,
) -> Result<(), ArtifactError> {
    if (!allow_empty && value.is_empty()) || value.contains('\r') || value.contains('\0') {
        return Err(ArtifactError::new(format!(
            "compiler diagnostic {label} is not canonical UTF-8/LF text"
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "compiler diagnostic {label} identity must be lowercase SHA-256"
        )))
    }
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler diagnostic report `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn diagnostic_record_identity(ordinal: usize, diagnostic: &CompilerDiagnosticInput<'_>) -> String {
    sha256_fields(&[
        DIAGNOSTIC_RECORD_IDENTITY_CONTRACT.as_bytes(),
        &(ordinal as u64).to_le_bytes(),
        diagnostic.module.as_bytes(),
        diagnostic.code.as_bytes(),
        diagnostic.path.as_bytes(),
        diagnostic.message.as_bytes(),
    ])
}

fn diagnostics_identity(
    accepted: bool,
    semantic_pipeline: &str,
    semantic_error: &str,
    diagnostics: &[CompilerDiagnosticRecord],
) -> String {
    let mut fields = vec![
        COMPILER_DIAGNOSTIC_NORMALIZATION_CONTRACT.as_bytes(),
        if accepted { b"accepted" } else { b"rejected" },
        semantic_pipeline.as_bytes(),
        semantic_error.as_bytes(),
    ];
    let count = (diagnostics.len() as u64).to_le_bytes();
    fields.push(&count);
    for diagnostic in diagnostics {
        fields.push(diagnostic.record_sha256.as_bytes());
    }
    sha256_fields(&fields)
}

fn report_identity(report: &CompilerDiagnosticReport) -> String {
    sha256_fields(&[
        report.protocol.as_bytes(),
        report.normalization_contract.as_bytes(),
        report.producer_id.as_bytes(),
        report.component_record_sha256.as_bytes(),
        report.bootstrap_subset_protocol.as_bytes(),
        report.diagnostics_sha256.as_bytes(),
    ])
}

fn sha256_fields(fields: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for field in fields {
        hash.update((field.len() as u64).to_le_bytes());
        hash.update(field);
    }
    let digest = hash.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
#[path = "compiler_diagnostic_report_tests.rs"]
mod tests;
