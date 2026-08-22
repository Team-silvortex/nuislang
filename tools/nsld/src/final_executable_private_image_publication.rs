use crate::{
    final_executable_finalizer_registry::ExecutableFinalizerPrivateImagePublicationContext,
    hash_sha256::sha256_hex, reports::NsldPrivateImagePublicationReport,
};
use sha2::{Digest, Sha256};
use std::{
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) const PRIVATE_IMAGE_PUBLICATION_CONTRACT: &str =
    "nuis-nsld-registered-private-image-publication-v1";

static PUBLICATION_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct PrivateImageAdmissionEvidence<'a> {
    pub(crate) contract: &'a str,
    pub(crate) status: &'a str,
    pub(crate) receipt_file: &'a str,
    pub(crate) receipt_present: bool,
    pub(crate) valid: bool,
    pub(crate) receipt_hash_sha256: Option<&'a str>,
    pub(crate) verification_ledger_sha256: &'a str,
    pub(crate) issues: &'a [String],
}

pub(crate) struct PrivateImagePublicationInput<'a, 'plan> {
    pub(crate) context: &'a ExecutableFinalizerPrivateImagePublicationContext<'plan>,
    pub(crate) admission: PrivateImageAdmissionEvidence<'a>,
    pub(crate) source_image: &'a [u8],
    pub(crate) source_image_hash: &'a str,
    pub(crate) provider_issues: Vec<String>,
}

pub(crate) fn publish_verified_private_image(
    input: PrivateImagePublicationInput<'_, '_>,
) -> Result<NsldPrivateImagePublicationReport, String> {
    let source_image_sha256 = sha256_hex(input.source_image);
    let output_before = output_snapshot(input.context.output_path)?;
    let mut issues = input
        .admission
        .issues
        .iter()
        .map(|issue| format!("publication-admission:{issue}"))
        .collect::<Vec<_>>();
    issues.extend(input.provider_issues);
    if input.context.output_path.file_name()
        != Path::new(&input.context.plan.compiled_artifact.binary_name).file_name()
    {
        issues.push("private-image-publication-output-name-mismatch".to_owned());
    }

    let publication_ready = input.admission.valid && issues.is_empty();
    let installation_attempted = input.context.apply && publication_ready;
    if installation_attempted {
        if let Err(error) =
            atomic_install_private_image(input.context.output_path, input.source_image)
        {
            issues.push(format!(
                "private-image-publication-installation-failed:{error}"
            ));
        }
    }

    let output_after = output_snapshot(input.context.output_path)?;
    let output_matches_private_image = output_after
        .sha256
        .as_deref()
        .is_some_and(|hash| hash == source_image_sha256)
        && output_after.span_bytes == Some(input.source_image.len());
    let installed = installation_attempted
        && issues.is_empty()
        && output_matches_private_image
        && output_after.executable;
    if installation_attempted && !output_matches_private_image {
        issues.push("private-image-publication-output-identity-mismatch".to_owned());
    }
    if installation_attempted && !output_after.executable {
        issues.push("private-image-publication-output-not-executable".to_owned());
    }
    let output_changed = output_before.present != output_after.present
        || output_before.sha256 != output_after.sha256;
    let status = if installed {
        "private-image-published"
    } else if input.context.apply && !publication_ready {
        "blocked-publication-admission-invalid"
    } else if input.context.apply {
        "private-image-publication-failed"
    } else if publication_ready {
        "ready-private-image-publication-plan"
    } else {
        "blocked-private-image-publication-plan"
    };
    let mut report = NsldPrivateImagePublicationReport {
        contract: PRIVATE_IMAGE_PUBLICATION_CONTRACT.to_owned(),
        status: status.to_owned(),
        provider_id: input.context.provider_id.to_owned(),
        target_key: input.context.target_key.to_owned(),
        capability_id: input.context.capability_id.to_owned(),
        apply_requested: input.context.apply,
        publication_ready,
        admission_contract: input.admission.contract.to_owned(),
        admission_status: input.admission.status.to_owned(),
        admission_receipt_file: input.admission.receipt_file.to_owned(),
        admission_receipt_present: input.admission.receipt_present,
        admission_receipt_valid: input.admission.valid,
        admission_receipt_hash_sha256: input.admission.receipt_hash_sha256.map(str::to_owned),
        admission_verification_ledger_sha256: input.admission.verification_ledger_sha256.to_owned(),
        source_image_span_bytes: input.source_image.len(),
        source_image_hash: input.source_image_hash.to_owned(),
        source_image_sha256,
        output_path: input.context.output_path.display().to_string(),
        output_present_before: output_before.present,
        output_sha256_before: output_before.sha256,
        installation_attempted,
        installed,
        output_present_after: output_after.present,
        output_span_bytes_after: output_after.span_bytes,
        output_sha256_after: output_after.sha256,
        output_matches_private_image,
        output_executable: output_after.executable,
        output_changed,
        issue_count: issues.len(),
        issues,
        publication_ledger_sha256: String::new(),
    };
    report.publication_ledger_sha256 = publication_ledger_sha256(&report);
    Ok(report)
}

#[derive(Default)]
struct OutputSnapshot {
    present: bool,
    span_bytes: Option<usize>,
    sha256: Option<String>,
    executable: bool,
}

fn output_snapshot(path: &Path) -> Result<OutputSnapshot, String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(OutputSnapshot::default());
        }
        Err(error) => {
            return Err(format!(
                "failed to inspect private-image publication output `{}`: {error}",
                path.display()
            ));
        }
    };
    if !metadata.is_file() {
        return Err(format!(
            "private-image publication output `{}` is not a regular file",
            path.display()
        ));
    }
    let (span_bytes, sha256) = sha256_file(path)?;
    Ok(OutputSnapshot {
        present: true,
        span_bytes: Some(span_bytes),
        sha256: Some(sha256),
        executable: output_is_executable(&metadata),
    })
}

fn atomic_install_private_image(output_path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create private-image publication directory `{}`: {error}",
            parent.display()
        )
    })?;
    let temp_path = private_image_temp_path(output_path);
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o700);
        }
        let mut file = options.open(&temp_path).map_err(|error| {
            format!(
                "failed to create temporary private image `{}`: {error}",
                temp_path.display()
            )
        })?;
        file.write_all(bytes).map_err(|error| {
            format!(
                "failed to write temporary private image `{}`: {error}",
                temp_path.display()
            )
        })?;
        set_owner_executable_permissions(&temp_path)?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync temporary private image `{}`: {error}",
                temp_path.display()
            )
        })?;
        drop(file);
        if !file_matches_bytes(&temp_path, bytes)? {
            return Err("temporary private-image byte identity drift".to_owned());
        }
        fs::rename(&temp_path, output_path).map_err(|error| {
            format!(
                "failed to atomically install private image `{}`: {error}",
                output_path.display()
            )
        })?;
        sync_directory(parent)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn sha256_file(path: &Path) -> Result<(usize, String), String> {
    let mut file = File::open(path).map_err(|error| {
        format!(
            "failed to read private-image publication output `{}`: {error}",
            path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut span_bytes = 0usize;
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut chunk).map_err(|error| {
            format!(
                "failed to hash private-image publication output `{}`: {error}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        span_bytes = span_bytes
            .checked_add(read)
            .ok_or_else(|| "private-image publication output size overflow".to_owned())?;
        hasher.update(&chunk[..read]);
    }
    let mut hash = String::with_capacity(64);
    for byte in hasher.finalize() {
        write!(hash, "{byte:02x}").unwrap();
    }
    Ok((span_bytes, hash))
}

fn file_matches_bytes(path: &Path, expected: &[u8]) -> Result<bool, String> {
    let mut file = File::open(path).map_err(|error| {
        format!(
            "failed to verify temporary private image `{}`: {error}",
            path.display()
        )
    })?;
    let mut offset = 0usize;
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut chunk).map_err(|error| {
            format!(
                "failed to compare temporary private image `{}`: {error}",
                path.display()
            )
        })?;
        if read == 0 {
            return Ok(offset == expected.len());
        }
        let end = offset.saturating_add(read);
        if expected.get(offset..end) != Some(&chunk[..read]) {
            return Ok(false);
        }
        offset = end;
    }
}

fn private_image_temp_path(output_path: &Path) -> PathBuf {
    let sequence = PUBLICATION_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-private-image");
    output_path.with_file_name(format!(
        ".{name}.nsld-private-publication-{}-{sequence}.tmp",
        std::process::id()
    ))
}

#[cfg(unix)]
fn set_owner_executable_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("failed to inspect `{}`: {error}", path.display()))?
        .permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).map_err(|error| {
        format!(
            "failed to secure `{}` as executable: {error}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
fn set_owner_executable_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn output_is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn output_is_executable(_metadata: &fs::Metadata) -> bool {
    true
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "failed to sync private-image publication directory `{}`: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn publication_ledger_sha256(report: &NsldPrivateImagePublicationReport) -> String {
    let mut material = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
        report.contract,
        report.status,
        report.provider_id,
        report.target_key,
        report.capability_id,
        report.apply_requested,
        report.publication_ready,
        report.admission_contract,
        report.admission_status,
        report.admission_receipt_file,
        report.admission_receipt_present,
        report.admission_receipt_valid,
        report
            .admission_receipt_hash_sha256
            .as_deref()
            .unwrap_or("none"),
        report.admission_verification_ledger_sha256,
        report.source_image_span_bytes,
        report.source_image_hash,
        report.source_image_sha256,
        report.output_path,
        report.output_present_before,
        report.output_sha256_before.as_deref().unwrap_or("none"),
        report.installation_attempted,
        report.installed,
        report.output_present_after,
        report.output_span_bytes_after.unwrap_or(0),
        report.output_sha256_after.as_deref().unwrap_or("none"),
        report.output_matches_private_image,
        report.output_executable,
    );
    writeln!(material, "output_changed={}", report.output_changed).unwrap();
    for issue in &report.issues {
        writeln!(material, "issue={issue}").unwrap();
    }
    sha256_hex(material.as_bytes())
}
