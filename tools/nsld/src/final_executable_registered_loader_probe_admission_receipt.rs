use crate::{
    final_executable_registered_loader_probe::{
        NsldRegisteredLoaderProbeOutcome, REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT,
    },
    hash_sha256::sha256_hex,
};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) const REGISTERED_LOADER_PROBE_ADMISSION_CONTRACT: &str =
    "nuis-nsld-registered-loader-probe-admission-v1";
pub(crate) const REGISTERED_LOADER_PROBE_ADMISSION_STATUS: &str =
    "admitted-registered-loader-probe-execution";
pub(crate) const REGISTERED_LOADER_PROBE_ADMISSION_FILE: &str =
    "nuis.nsld.registered-loader-probe-admission.toml";

static RECEIPT_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldRegisteredLoaderProbeAdmissionReceipt {
    pub(crate) contract: String,
    pub(crate) status: String,
    pub(crate) finalizer_registry_contract: String,
    pub(crate) finalizer_registry_hash: String,
    pub(crate) finalizer_provider_id: String,
    pub(crate) finalizer_target_key: String,
    pub(crate) loader_probe_capability_id: String,
    pub(crate) target_abi: String,
    pub(crate) machine_arch: String,
    pub(crate) machine_os: String,
    pub(crate) object_format: String,
    pub(crate) calling_abi: String,
    pub(crate) packaging_mode: String,
    pub(crate) outcome: NsldRegisteredLoaderProbeOutcome,
    pub(crate) receipt_hash_sha256: String,
}

pub(crate) fn registered_loader_probe_admission_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    Path::new(&plan.output_dir).join(REGISTERED_LOADER_PROBE_ADMISSION_FILE)
}

pub(crate) fn persist_registered_loader_probe_admission_receipt(
    plan: &nuisc::linker::LinkPlan,
    receipt: &NsldRegisteredLoaderProbeAdmissionReceipt,
) -> Result<PathBuf, String> {
    if receipt.receipt_hash_sha256 != registered_loader_probe_admission_receipt_hash(receipt)? {
        return Err("registered loader-probe admission refuses receipt hash drift".to_owned());
    }
    let source = render_registered_loader_probe_admission_receipt(receipt)?;
    if parse_registered_loader_probe_admission_receipt(&source)? != *receipt {
        return Err(
            "registered loader-probe admission receipt does not survive canonical roundtrip"
                .to_owned(),
        );
    }
    let path = registered_loader_probe_admission_path(plan);
    atomic_write_receipt(&path, source.as_bytes())?;
    Ok(path)
}

pub(crate) fn render_registered_loader_probe_admission_receipt(
    receipt: &NsldRegisteredLoaderProbeAdmissionReceipt,
) -> Result<String, String> {
    if !valid_receipt_string(&receipt.receipt_hash_sha256) {
        return Err("registered loader-probe admission receipt hash is not canonical".to_owned());
    }
    let mut out = receipt_payload(receipt)?;
    string_line(
        &mut out,
        "receipt_hash_sha256",
        &receipt.receipt_hash_sha256,
    );
    Ok(out)
}

pub(crate) fn parse_registered_loader_probe_admission_receipt(
    source: &str,
) -> Result<NsldRegisteredLoaderProbeAdmissionReceipt, String> {
    let mut fields = StrictReceiptFields::parse(source)?;
    let contract = fields.string("contract")?;
    let status = fields.string("status")?;
    let finalizer_registry_contract = fields.string("finalizer_registry_contract")?;
    let finalizer_registry_hash = fields.string("finalizer_registry_hash")?;
    let finalizer_provider_id = fields.string("finalizer_provider_id")?;
    let finalizer_target_key = fields.string("finalizer_target_key")?;
    let loader_probe_capability_id = fields.string("loader_probe_capability_id")?;
    let target_abi = fields.string("target_abi")?;
    let machine_arch = fields.string("machine_arch")?;
    let machine_os = fields.string("machine_os")?;
    let object_format = fields.string("object_format")?;
    let calling_abi = fields.string("calling_abi")?;
    let packaging_mode = fields.string("packaging_mode")?;
    let outcome_contract = fields.string("outcome_contract")?;
    let outcome_status = fields.string("outcome_status")?;
    if outcome_contract != REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT
        || outcome_status != "execution-admitted"
    {
        return Err("registered-loader-probe-admission-outcome-contract-invalid".to_owned());
    }
    let outcome = NsldRegisteredLoaderProbeOutcome {
        contract: REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT,
        status: "execution-admitted",
        provider_id: finalizer_provider_id.clone(),
        target_key: finalizer_target_key.clone(),
        capability_id: loader_probe_capability_id.clone(),
        provider_probe_contract: fields.string("provider_probe_contract")?,
        provider_probe_status: fields.string("provider_probe_status")?,
        probe_mode: fields.string("probe_mode")?,
        host_supported: fields.boolean("host_supported")?,
        input_eligible: fields.boolean("input_eligible")?,
        attempted: fields.boolean("attempted")?,
        image_span_bytes: fields.usize("image_span_bytes")?,
        image_identity_hash: fields.string("image_identity_hash")?,
        validation_evidence_hash: fields.string("validation_evidence_hash")?,
        materialized: fields.boolean("materialized")?,
        materialized_hash_matches: fields.boolean("materialized_hash_matches")?,
        os_loader_accepted: fields.boolean("os_loader_accepted")?,
        process_completed: fields.boolean("process_completed")?,
        timed_out: fields.boolean("timed_out")?,
        exit_code: fields.optional_i32("exit_code")?,
        termination_signal: fields.optional_i32("termination_signal")?,
        stdout_captured_bytes: fields.usize("stdout_captured_bytes")?,
        stdout_truncated: fields.boolean("stdout_truncated")?,
        stderr_captured_bytes: fields.usize("stderr_captured_bytes")?,
        stderr_truncated: fields.boolean("stderr_truncated")?,
        failure_kind: fields.optional_string("failure_kind")?,
        cleanup_attempted: fields.boolean("cleanup_attempted")?,
        cleanup_succeeded: fields.boolean("cleanup_succeeded")?,
        execution_admitted: fields.boolean("execution_admitted")?,
        blockers: Vec::new(),
        provider_evidence_hash: fields.string("provider_evidence_hash")?,
        outcome_ledger_hash: fields.string("outcome_ledger_hash")?,
    };
    if fields.usize("blocker_count")? != 0 {
        return Err("registered-loader-probe-admission-blockers-not-empty".to_owned());
    }
    let receipt_hash_sha256 = fields.string("receipt_hash_sha256")?;
    fields.finish()?;
    Ok(NsldRegisteredLoaderProbeAdmissionReceipt {
        contract,
        status,
        finalizer_registry_contract,
        finalizer_registry_hash,
        finalizer_provider_id,
        finalizer_target_key,
        loader_probe_capability_id,
        target_abi,
        machine_arch,
        machine_os,
        object_format,
        calling_abi,
        packaging_mode,
        outcome,
        receipt_hash_sha256,
    })
}

pub(crate) fn registered_loader_probe_admission_receipt_hash(
    receipt: &NsldRegisteredLoaderProbeAdmissionReceipt,
) -> Result<String, String> {
    receipt_payload(receipt).map(|payload| sha256_hex(payload.as_bytes()))
}

fn receipt_payload(receipt: &NsldRegisteredLoaderProbeAdmissionReceipt) -> Result<String, String> {
    validate_receipt_strings(receipt)?;
    let outcome = &receipt.outcome;
    let mut out = String::with_capacity(4096);
    string_line(&mut out, "contract", &receipt.contract);
    string_line(&mut out, "status", &receipt.status);
    string_line(
        &mut out,
        "finalizer_registry_contract",
        &receipt.finalizer_registry_contract,
    );
    string_line(
        &mut out,
        "finalizer_registry_hash",
        &receipt.finalizer_registry_hash,
    );
    string_line(
        &mut out,
        "finalizer_provider_id",
        &receipt.finalizer_provider_id,
    );
    string_line(
        &mut out,
        "finalizer_target_key",
        &receipt.finalizer_target_key,
    );
    string_line(
        &mut out,
        "loader_probe_capability_id",
        &receipt.loader_probe_capability_id,
    );
    string_line(&mut out, "target_abi", &receipt.target_abi);
    string_line(&mut out, "machine_arch", &receipt.machine_arch);
    string_line(&mut out, "machine_os", &receipt.machine_os);
    string_line(&mut out, "object_format", &receipt.object_format);
    string_line(&mut out, "calling_abi", &receipt.calling_abi);
    string_line(&mut out, "packaging_mode", &receipt.packaging_mode);
    string_line(&mut out, "outcome_contract", outcome.contract);
    string_line(&mut out, "outcome_status", outcome.status);
    string_line(
        &mut out,
        "provider_probe_contract",
        &outcome.provider_probe_contract,
    );
    string_line(
        &mut out,
        "provider_probe_status",
        &outcome.provider_probe_status,
    );
    string_line(&mut out, "probe_mode", &outcome.probe_mode);
    bool_line(&mut out, "host_supported", outcome.host_supported);
    bool_line(&mut out, "input_eligible", outcome.input_eligible);
    bool_line(&mut out, "attempted", outcome.attempted);
    usize_line(&mut out, "image_span_bytes", outcome.image_span_bytes);
    string_line(
        &mut out,
        "image_identity_hash",
        &outcome.image_identity_hash,
    );
    string_line(
        &mut out,
        "validation_evidence_hash",
        &outcome.validation_evidence_hash,
    );
    bool_line(&mut out, "materialized", outcome.materialized);
    bool_line(
        &mut out,
        "materialized_hash_matches",
        outcome.materialized_hash_matches,
    );
    bool_line(&mut out, "os_loader_accepted", outcome.os_loader_accepted);
    bool_line(&mut out, "process_completed", outcome.process_completed);
    bool_line(&mut out, "timed_out", outcome.timed_out);
    option_i32_line(&mut out, "exit_code", outcome.exit_code);
    option_i32_line(&mut out, "termination_signal", outcome.termination_signal);
    usize_line(
        &mut out,
        "stdout_captured_bytes",
        outcome.stdout_captured_bytes,
    );
    bool_line(&mut out, "stdout_truncated", outcome.stdout_truncated);
    usize_line(
        &mut out,
        "stderr_captured_bytes",
        outcome.stderr_captured_bytes,
    );
    bool_line(&mut out, "stderr_truncated", outcome.stderr_truncated);
    option_string_line(&mut out, "failure_kind", outcome.failure_kind.as_deref());
    bool_line(&mut out, "cleanup_attempted", outcome.cleanup_attempted);
    bool_line(&mut out, "cleanup_succeeded", outcome.cleanup_succeeded);
    bool_line(&mut out, "execution_admitted", outcome.execution_admitted);
    usize_line(&mut out, "blocker_count", outcome.blockers.len());
    string_line(
        &mut out,
        "provider_evidence_hash",
        &outcome.provider_evidence_hash,
    );
    string_line(
        &mut out,
        "outcome_ledger_hash",
        &outcome.outcome_ledger_hash,
    );
    Ok(out)
}

fn validate_receipt_strings(
    receipt: &NsldRegisteredLoaderProbeAdmissionReceipt,
) -> Result<(), String> {
    let outcome = &receipt.outcome;
    let values = [
        receipt.contract.as_str(),
        receipt.status.as_str(),
        receipt.finalizer_registry_contract.as_str(),
        receipt.finalizer_registry_hash.as_str(),
        receipt.finalizer_provider_id.as_str(),
        receipt.finalizer_target_key.as_str(),
        receipt.loader_probe_capability_id.as_str(),
        receipt.target_abi.as_str(),
        receipt.machine_arch.as_str(),
        receipt.machine_os.as_str(),
        receipt.object_format.as_str(),
        receipt.calling_abi.as_str(),
        receipt.packaging_mode.as_str(),
        outcome.contract,
        outcome.status,
        outcome.provider_probe_contract.as_str(),
        outcome.provider_probe_status.as_str(),
        outcome.probe_mode.as_str(),
        outcome.image_identity_hash.as_str(),
        outcome.validation_evidence_hash.as_str(),
        outcome.provider_evidence_hash.as_str(),
        outcome.outcome_ledger_hash.as_str(),
    ];
    if values.into_iter().any(|value| !valid_receipt_string(value))
        || outcome
            .failure_kind
            .as_deref()
            .is_some_and(|value| !valid_receipt_string(value))
        || !outcome.blockers.is_empty()
    {
        return Err(
            "registered loader-probe admission receipt contains non-canonical evidence".to_owned(),
        );
    }
    Ok(())
}

fn valid_receipt_string(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/' | b'+')
        })
}

fn string_line(out: &mut String, key: &str, value: &str) {
    writeln!(out, "{key} = \"{value}\"").unwrap();
}

fn usize_line(out: &mut String, key: &str, value: usize) {
    writeln!(out, "{key} = {value}").unwrap();
}

fn bool_line(out: &mut String, key: &str, value: bool) {
    writeln!(out, "{key} = {value}").unwrap();
}

fn option_i32_line(out: &mut String, key: &str, value: Option<i32>) {
    match value {
        Some(value) => writeln!(out, "{key} = {value}").unwrap(),
        None => string_line(out, key, "none"),
    }
}

fn option_string_line(out: &mut String, key: &str, value: Option<&str>) {
    string_line(out, key, value.unwrap_or("none"));
}

struct StrictReceiptFields {
    values: BTreeMap<String, String>,
}

impl StrictReceiptFields {
    fn parse(source: &str) -> Result<Self, String> {
        let mut values = BTreeMap::new();
        for (index, raw) in source.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=').ok_or_else(|| {
                format!(
                    "registered-loader-probe-admission-line-{}-malformed",
                    index + 1
                )
            })?;
            let key = key.trim();
            if key.is_empty()
                || !key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(format!(
                    "registered-loader-probe-admission-line-{}-key-invalid",
                    index + 1
                ));
            }
            if values
                .insert(key.to_owned(), value.trim().to_owned())
                .is_some()
            {
                return Err(format!(
                    "registered-loader-probe-admission-duplicate-key:{key}"
                ));
            }
        }
        Ok(Self { values })
    }

    fn string(&mut self, key: &str) -> Result<String, String> {
        let raw = self.take(key)?;
        let value = raw
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .ok_or_else(|| format!("registered-loader-probe-admission-string-invalid:{key}"))?;
        if !valid_receipt_string(value) {
            return Err(format!(
                "registered-loader-probe-admission-string-invalid:{key}"
            ));
        }
        Ok(value.to_owned())
    }

    fn optional_string(&mut self, key: &str) -> Result<Option<String>, String> {
        self.string(key)
            .map(|value| (value != "none").then_some(value))
    }

    fn usize(&mut self, key: &str) -> Result<usize, String> {
        self.take(key)?
            .parse()
            .map_err(|_| format!("registered-loader-probe-admission-usize-invalid:{key}"))
    }

    fn boolean(&mut self, key: &str) -> Result<bool, String> {
        self.take(key)?
            .parse()
            .map_err(|_| format!("registered-loader-probe-admission-bool-invalid:{key}"))
    }

    fn optional_i32(&mut self, key: &str) -> Result<Option<i32>, String> {
        let raw = self.take(key)?;
        if raw == "\"none\"" {
            Ok(None)
        } else {
            raw.parse()
                .map(Some)
                .map_err(|_| format!("registered-loader-probe-admission-i32-invalid:{key}"))
        }
    }

    fn finish(self) -> Result<(), String> {
        if self.values.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "registered-loader-probe-admission-unknown-keys:{}",
                self.values.keys().cloned().collect::<Vec<_>>().join(",")
            ))
        }
    }

    fn take(&mut self, key: &str) -> Result<String, String> {
        self.values
            .remove(key)
            .ok_or_else(|| format!("registered-loader-probe-admission-key-missing:{key}"))
    }
}

fn atomic_write_receipt(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create registered loader-probe admission directory `{}`: {error}",
            parent.display()
        )
    })?;
    let sequence = RECEIPT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_file_name(format!(
        ".{REGISTERED_LOADER_PROBE_ADMISSION_FILE}.{}-{sequence}.tmp",
        std::process::id()
    ));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp).map_err(|error| {
            format!(
                "failed to create temporary registered loader-probe admission receipt `{}`: {error}",
                temp.display()
            )
        })?;
        file.write_all(bytes).map_err(|error| {
            format!(
                "failed to write temporary registered loader-probe admission receipt `{}`: {error}",
                temp.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync temporary registered loader-probe admission receipt `{}`: {error}",
                temp.display()
            )
        })?;
        drop(file);
        fs::rename(&temp, path).map_err(|error| {
            format!(
                "failed to atomically install registered loader-probe admission receipt `{}`: {error}",
                path.display()
            )
        })?;
        sync_directory(parent)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "failed to sync registered loader-probe admission directory `{}`: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
#[path = "final_executable_registered_loader_probe_admission_receipt_tests.rs"]
mod tests;
