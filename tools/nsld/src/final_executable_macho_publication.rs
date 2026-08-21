use crate::{
    final_executable_finalizer_registry::ExecutableFinalizerPrivateImagePublicationContext,
    final_executable_macho_admission::verify_macho_arm64_publication_admission_receipt,
    final_executable_macho_artifact::macho_artifact_private_shell_product,
    final_executable_macho_shell_signature::sha256_hex, reports::NsldPrivateImagePublicationReport,
};
use sha2::{Digest, Sha256};
use std::{
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{Read as _, Write as _},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) const MACHO_ARM64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY: &str =
    "nsld.finalizer.mach-o.arm64.private-image-publication-v1";
pub(crate) const PRIVATE_IMAGE_PUBLICATION_CONTRACT: &str =
    "nuis-nsld-registered-private-image-publication-v1";
static PUBLICATION_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn publish_macho_arm64_private_image(
    context: &ExecutableFinalizerPrivateImagePublicationContext<'_>,
) -> Result<NsldPrivateImagePublicationReport, String> {
    let product = macho_artifact_private_shell_product(context.plan)?;
    let admission = verify_macho_arm64_publication_admission_receipt(context.plan, &product);
    let source_image_hash = product
        .summary
        .shell_image_serialization
        .shell_image_hash
        .clone();
    let source_image_sha256 = sha256_hex(&product.bytes);
    let output_before = output_snapshot(context.output_path)?;
    let mut issues = admission
        .issues
        .iter()
        .map(|issue| format!("publication-admission:{issue}"))
        .collect::<Vec<_>>();

    if context.capability_id != MACHO_ARM64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY {
        issues.push("private-image-publication-capability-identity-mismatch".to_owned());
    }
    if context.target_key != "aarch64-macos-mach-o" {
        issues.push("private-image-publication-target-identity-mismatch".to_owned());
    }
    if context.output_path.file_name()
        != Path::new(&context.plan.compiled_artifact.binary_name).file_name()
    {
        issues.push("private-image-publication-output-name-mismatch".to_owned());
    }

    let publication_ready = admission.valid && issues.is_empty();
    let installation_attempted = context.apply && publication_ready;
    if installation_attempted {
        if let Err(error) = atomic_install_private_image(context.output_path, &product.bytes) {
            issues.push(format!(
                "private-image-publication-installation-failed:{error}"
            ));
        }
    }

    let output_after = output_snapshot(context.output_path)?;
    let output_matches_private_image = output_after
        .sha256
        .as_deref()
        .is_some_and(|hash| hash == source_image_sha256)
        && output_after.span_bytes == Some(product.bytes.len());
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
    } else if context.apply && !publication_ready {
        "blocked-publication-admission-invalid"
    } else if context.apply {
        "private-image-publication-failed"
    } else if publication_ready {
        "ready-private-image-publication-plan"
    } else {
        "blocked-private-image-publication-plan"
    };
    let mut report = NsldPrivateImagePublicationReport {
        contract: PRIVATE_IMAGE_PUBLICATION_CONTRACT.to_owned(),
        status: status.to_owned(),
        provider_id: context.provider_id.to_owned(),
        target_key: context.target_key.to_owned(),
        capability_id: context.capability_id.to_owned(),
        apply_requested: context.apply,
        publication_ready,
        admission_contract: admission.contract,
        admission_status: admission.status,
        admission_receipt_file: admission.receipt_file,
        admission_receipt_present: admission.receipt_present,
        admission_receipt_valid: admission.valid,
        admission_receipt_hash_sha256: admission.receipt_hash_sha256,
        admission_verification_ledger_sha256: admission.verification_ledger_sha256,
        source_image_span_bytes: product.bytes.len(),
        source_image_hash,
        source_image_sha256,
        output_path: context.output_path.display().to_string(),
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
