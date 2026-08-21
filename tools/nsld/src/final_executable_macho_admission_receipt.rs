use crate::{
    final_executable_macho_shell_signature::sha256_hex,
    reports::NsldMachOArm64PublicationAdmissionReceipt,
};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) const MACHO_ARM64_PUBLICATION_ADMISSION_CONTRACT: &str =
    "nuis-nsld-macho-arm64-publication-admission-v1";
pub(crate) const MACHO_ARM64_PUBLICATION_ADMISSION_FILE: &str =
    "nuis.nsld.macho-arm64-publication-admission.toml";
pub(crate) const MACHO_ARM64_PUBLICATION_ADMISSION_STATUS: &str =
    "admitted-loader-probe-bound-private-image";
static RECEIPT_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn persist_macho_arm64_publication_admission_receipt(
    plan: &nuisc::linker::LinkPlan,
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> Result<PathBuf, String> {
    if receipt.receipt_hash_sha256 != receipt_hash_sha256(receipt)? {
        return Err("Mach-O admission refuses to persist receipt hash drift".to_owned());
    }
    let source = render_macho_arm64_publication_admission_receipt(receipt)?;
    let parsed = parse_macho_arm64_publication_admission_receipt(&source)?;
    if parsed != *receipt {
        return Err("Mach-O admission receipt does not survive canonical roundtrip".to_owned());
    }
    let path = macho_arm64_publication_admission_path(plan);
    atomic_write_receipt(&path, source.as_bytes())?;
    Ok(path)
}

pub(crate) fn macho_arm64_publication_admission_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    Path::new(&plan.output_dir).join(MACHO_ARM64_PUBLICATION_ADMISSION_FILE)
}

pub(crate) fn render_macho_arm64_publication_admission_receipt(
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> Result<String, String> {
    if !valid_receipt_string(&receipt.receipt_hash_sha256) {
        return Err("Mach-O admission receipt hash is not canonical".to_owned());
    }
    let mut out = receipt_payload(receipt)?;
    string_line(
        &mut out,
        "receipt_hash_sha256",
        &receipt.receipt_hash_sha256,
    );
    Ok(out)
}

pub(crate) fn parse_macho_arm64_publication_admission_receipt(
    source: &str,
) -> Result<NsldMachOArm64PublicationAdmissionReceipt, String> {
    let mut fields = StrictReceiptFields::parse(source)?;
    let receipt = NsldMachOArm64PublicationAdmissionReceipt {
        contract: fields.string("contract")?,
        status: fields.string("status")?,
        finalizer_registry_contract: fields.string("finalizer_registry_contract")?,
        finalizer_registry_hash: fields.string("finalizer_registry_hash")?,
        finalizer_provider_id: fields.string("finalizer_provider_id")?,
        finalizer_target_key: fields.string("finalizer_target_key")?,
        target_arch: fields.string("target_arch")?,
        target_os: fields.string("target_os")?,
        object_format: fields.string("object_format")?,
        calling_abi: fields.string("calling_abi")?,
        packaging_mode: fields.string("packaging_mode")?,
        object_linkage_hash: fields.string("object_linkage_hash")?,
        shell_layout_plan_hash: fields.string("shell_layout_plan_hash")?,
        serialization_ledger_hash: fields.string("serialization_ledger_hash")?,
        shell_image_span_bytes: fields.usize("shell_image_span_bytes")?,
        shell_image_hash: fields.string("shell_image_hash")?,
        shell_image_sha256: fields.string("shell_image_sha256")?,
        signature_validation_contract: fields.string("signature_validation_contract")?,
        signature_validation_status: fields.string("signature_validation_status")?,
        signature_validation_ledger_hash: fields.string("signature_validation_ledger_hash")?,
        signature_cdhash: fields.string("signature_cdhash")?,
        probe_contract: fields.string("probe_contract")?,
        probe_status: fields.string("probe_status")?,
        probe_ledger_hash: fields.string("probe_ledger_hash")?,
        probe_timeout_millis: fields.u64("probe_timeout_millis")?,
        probe_host_supported: fields.boolean("probe_host_supported")?,
        probe_input_eligible: fields.boolean("probe_input_eligible")?,
        probe_attempted: fields.boolean("probe_attempted")?,
        probe_materialized: fields.boolean("probe_materialized")?,
        probe_materialized_hash_matches: fields.boolean("probe_materialized_hash_matches")?,
        probe_kernel_accepted: fields.boolean("probe_kernel_accepted")?,
        probe_process_completed: fields.boolean("probe_process_completed")?,
        probe_timed_out: fields.boolean("probe_timed_out")?,
        probe_exit_code: fields.optional_i32("probe_exit_code")?,
        probe_termination_signal: fields.optional_i32("probe_termination_signal")?,
        probe_stdout_captured_bytes: fields.usize("probe_stdout_captured_bytes")?,
        probe_stdout_truncated: fields.boolean("probe_stdout_truncated")?,
        probe_stdout_hash: fields.string("probe_stdout_hash")?,
        probe_stderr_captured_bytes: fields.usize("probe_stderr_captured_bytes")?,
        probe_stderr_truncated: fields.boolean("probe_stderr_truncated")?,
        probe_stderr_hash: fields.string("probe_stderr_hash")?,
        probe_failure_kind: fields.optional_string("probe_failure_kind")?,
        probe_cleanup_attempted: fields.boolean("probe_cleanup_attempted")?,
        probe_cleanup_succeeded: fields.boolean("probe_cleanup_succeeded")?,
        unresolved_external_symbol_count: fields.usize("unresolved_external_symbol_count")?,
        bind_count: fields.usize("bind_count")?,
        publication_eligibility_contract: fields.string("publication_eligibility_contract")?,
        publication_eligibility_status: fields.string("publication_eligibility_status")?,
        publication_eligible: fields.boolean("publication_eligible")?,
        receipt_hash_sha256: fields.string("receipt_hash_sha256")?,
    };
    fields.finish()?;
    Ok(receipt)
}

pub(crate) fn receipt_hash_sha256(
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> Result<String, String> {
    receipt_payload(receipt).map(|payload| sha256_hex(payload.as_bytes()))
}

fn receipt_payload(receipt: &NsldMachOArm64PublicationAdmissionReceipt) -> Result<String, String> {
    validate_receipt_strings(receipt)?;
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
    string_line(&mut out, "target_arch", &receipt.target_arch);
    string_line(&mut out, "target_os", &receipt.target_os);
    string_line(&mut out, "object_format", &receipt.object_format);
    string_line(&mut out, "calling_abi", &receipt.calling_abi);
    string_line(&mut out, "packaging_mode", &receipt.packaging_mode);
    string_line(
        &mut out,
        "object_linkage_hash",
        &receipt.object_linkage_hash,
    );
    string_line(
        &mut out,
        "shell_layout_plan_hash",
        &receipt.shell_layout_plan_hash,
    );
    string_line(
        &mut out,
        "serialization_ledger_hash",
        &receipt.serialization_ledger_hash,
    );
    usize_line(
        &mut out,
        "shell_image_span_bytes",
        receipt.shell_image_span_bytes,
    );
    string_line(&mut out, "shell_image_hash", &receipt.shell_image_hash);
    string_line(&mut out, "shell_image_sha256", &receipt.shell_image_sha256);
    string_line(
        &mut out,
        "signature_validation_contract",
        &receipt.signature_validation_contract,
    );
    string_line(
        &mut out,
        "signature_validation_status",
        &receipt.signature_validation_status,
    );
    string_line(
        &mut out,
        "signature_validation_ledger_hash",
        &receipt.signature_validation_ledger_hash,
    );
    string_line(&mut out, "signature_cdhash", &receipt.signature_cdhash);
    string_line(&mut out, "probe_contract", &receipt.probe_contract);
    string_line(&mut out, "probe_status", &receipt.probe_status);
    string_line(&mut out, "probe_ledger_hash", &receipt.probe_ledger_hash);
    u64_line(
        &mut out,
        "probe_timeout_millis",
        receipt.probe_timeout_millis,
    );
    bool_line(
        &mut out,
        "probe_host_supported",
        receipt.probe_host_supported,
    );
    bool_line(
        &mut out,
        "probe_input_eligible",
        receipt.probe_input_eligible,
    );
    bool_line(&mut out, "probe_attempted", receipt.probe_attempted);
    bool_line(&mut out, "probe_materialized", receipt.probe_materialized);
    bool_line(
        &mut out,
        "probe_materialized_hash_matches",
        receipt.probe_materialized_hash_matches,
    );
    bool_line(
        &mut out,
        "probe_kernel_accepted",
        receipt.probe_kernel_accepted,
    );
    bool_line(
        &mut out,
        "probe_process_completed",
        receipt.probe_process_completed,
    );
    bool_line(&mut out, "probe_timed_out", receipt.probe_timed_out);
    option_i32_line(&mut out, "probe_exit_code", receipt.probe_exit_code);
    option_i32_line(
        &mut out,
        "probe_termination_signal",
        receipt.probe_termination_signal,
    );
    usize_line(
        &mut out,
        "probe_stdout_captured_bytes",
        receipt.probe_stdout_captured_bytes,
    );
    bool_line(
        &mut out,
        "probe_stdout_truncated",
        receipt.probe_stdout_truncated,
    );
    string_line(&mut out, "probe_stdout_hash", &receipt.probe_stdout_hash);
    usize_line(
        &mut out,
        "probe_stderr_captured_bytes",
        receipt.probe_stderr_captured_bytes,
    );
    bool_line(
        &mut out,
        "probe_stderr_truncated",
        receipt.probe_stderr_truncated,
    );
    string_line(&mut out, "probe_stderr_hash", &receipt.probe_stderr_hash);
    option_string_line(
        &mut out,
        "probe_failure_kind",
        receipt.probe_failure_kind.as_deref(),
    );
    bool_line(
        &mut out,
        "probe_cleanup_attempted",
        receipt.probe_cleanup_attempted,
    );
    bool_line(
        &mut out,
        "probe_cleanup_succeeded",
        receipt.probe_cleanup_succeeded,
    );
    usize_line(
        &mut out,
        "unresolved_external_symbol_count",
        receipt.unresolved_external_symbol_count,
    );
    usize_line(&mut out, "bind_count", receipt.bind_count);
    string_line(
        &mut out,
        "publication_eligibility_contract",
        &receipt.publication_eligibility_contract,
    );
    string_line(
        &mut out,
        "publication_eligibility_status",
        &receipt.publication_eligibility_status,
    );
    bool_line(
        &mut out,
        "publication_eligible",
        receipt.publication_eligible,
    );
    Ok(out)
}

fn validate_receipt_strings(
    receipt: &NsldMachOArm64PublicationAdmissionReceipt,
) -> Result<(), String> {
    let values = [
        receipt.contract.as_str(),
        receipt.status.as_str(),
        receipt.finalizer_registry_contract.as_str(),
        receipt.finalizer_registry_hash.as_str(),
        receipt.finalizer_provider_id.as_str(),
        receipt.finalizer_target_key.as_str(),
        receipt.target_arch.as_str(),
        receipt.target_os.as_str(),
        receipt.object_format.as_str(),
        receipt.calling_abi.as_str(),
        receipt.packaging_mode.as_str(),
        receipt.object_linkage_hash.as_str(),
        receipt.shell_layout_plan_hash.as_str(),
        receipt.serialization_ledger_hash.as_str(),
        receipt.shell_image_hash.as_str(),
        receipt.shell_image_sha256.as_str(),
        receipt.signature_validation_contract.as_str(),
        receipt.signature_validation_status.as_str(),
        receipt.signature_validation_ledger_hash.as_str(),
        receipt.signature_cdhash.as_str(),
        receipt.probe_contract.as_str(),
        receipt.probe_status.as_str(),
        receipt.probe_ledger_hash.as_str(),
        receipt.probe_stdout_hash.as_str(),
        receipt.probe_stderr_hash.as_str(),
        receipt.publication_eligibility_contract.as_str(),
        receipt.publication_eligibility_status.as_str(),
    ];
    if values.into_iter().any(|value| !valid_receipt_string(value))
        || receipt
            .probe_failure_kind
            .as_deref()
            .is_some_and(|value| !valid_receipt_string(value))
    {
        return Err("Mach-O admission receipt contains a non-canonical string".to_owned());
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

fn u64_line(out: &mut String, key: &str, value: u64) {
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
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("publication-admission-line-{}-malformed", index + 1))?;
            let key = key.trim();
            if key.is_empty()
                || !key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(format!(
                    "publication-admission-line-{}-key-invalid",
                    index + 1
                ));
            }
            if values
                .insert(key.to_owned(), value.trim().to_owned())
                .is_some()
            {
                return Err(format!("publication-admission-duplicate-key:{key}"));
            }
        }
        Ok(Self { values })
    }

    fn string(&mut self, key: &str) -> Result<String, String> {
        let raw = self.take(key)?;
        let value = raw
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .ok_or_else(|| format!("publication-admission-string-invalid:{key}"))?;
        if !valid_receipt_string(value) {
            return Err(format!("publication-admission-string-invalid:{key}"));
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
            .map_err(|_| format!("publication-admission-usize-invalid:{key}"))
    }

    fn u64(&mut self, key: &str) -> Result<u64, String> {
        self.take(key)?
            .parse()
            .map_err(|_| format!("publication-admission-u64-invalid:{key}"))
    }

    fn boolean(&mut self, key: &str) -> Result<bool, String> {
        self.take(key)?
            .parse()
            .map_err(|_| format!("publication-admission-bool-invalid:{key}"))
    }

    fn optional_i32(&mut self, key: &str) -> Result<Option<i32>, String> {
        let raw = self.take(key)?;
        if raw == "\"none\"" {
            Ok(None)
        } else {
            raw.parse()
                .map(Some)
                .map_err(|_| format!("publication-admission-i32-invalid:{key}"))
        }
    }

    fn finish(self) -> Result<(), String> {
        if self.values.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "publication-admission-unknown-keys:{}",
                self.values.keys().cloned().collect::<Vec<_>>().join(",")
            ))
        }
    }

    fn take(&mut self, key: &str) -> Result<String, String> {
        self.values
            .remove(key)
            .ok_or_else(|| format!("publication-admission-key-missing:{key}"))
    }
}

fn atomic_write_receipt(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create Mach-O admission directory `{}`: {error}",
            parent.display()
        )
    })?;
    let sequence = RECEIPT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_file_name(format!(
        ".{MACHO_ARM64_PUBLICATION_ADMISSION_FILE}.{}-{sequence}.tmp",
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
                "failed to create temporary Mach-O admission receipt `{}`: {error}",
                temp.display()
            )
        })?;
        file.write_all(bytes).map_err(|error| {
            format!(
                "failed to write temporary Mach-O admission receipt `{}`: {error}",
                temp.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync temporary Mach-O admission receipt `{}`: {error}",
                temp.display()
            )
        })?;
        drop(file);
        fs::rename(&temp, path).map_err(|error| {
            format!(
                "failed to atomically install Mach-O admission receipt `{}`: {error}",
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
                "failed to sync Mach-O admission directory `{}`: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}
