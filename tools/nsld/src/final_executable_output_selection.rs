use crate::{
    final_executable_finalizer_registry::invoke_registered_private_image_publication,
    reports::{NsldFinalOutputSelectionReport, NsldPrivateImagePublicationReport},
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Read as _,
    path::Path,
};

pub(crate) const FINAL_OUTPUT_SELECTION_REGISTRY_CONTRACT: &str =
    "nuis-nsld-final-output-selection-registry-v1";
pub(crate) const FINAL_OUTPUT_SELECTION_EVIDENCE_CONTRACT: &str =
    "nuis-nsld-final-output-selection-evidence-v1";
pub(crate) const COMPATIBILITY_OUTPUT_POLICY: &str = "compatibility-default";
pub(crate) const ADMITTED_PRIVATE_IMAGE_OUTPUT_POLICY: &str = "admitted-private-image";

type SelectionCallback =
    for<'a> fn(&FinalOutputSelectionContext<'a>) -> Result<NsldFinalOutputSelectionReport, String>;

#[derive(Clone, Copy)]
struct FinalOutputSelectionPolicyRegistration {
    policy_id: &'static str,
    policy_status: &'static str,
    selection_kind: &'static str,
    default_policy: bool,
    requires_explicit_request: bool,
    supports_apply: bool,
    selector: SelectionCallback,
}

const REGISTERED_SELECTION_POLICIES: &[FinalOutputSelectionPolicyRegistration] = &[
    FinalOutputSelectionPolicyRegistration {
        policy_id: COMPATIBILITY_OUTPUT_POLICY,
        policy_status: "ready",
        selection_kind: "existing-compatibility-output",
        default_policy: true,
        requires_explicit_request: false,
        supports_apply: false,
        selector: preserve_compatibility_output,
    },
    FinalOutputSelectionPolicyRegistration {
        policy_id: ADMITTED_PRIVATE_IMAGE_OUTPUT_POLICY,
        policy_status: "ready",
        selection_kind: "registered-admitted-private-image",
        default_policy: false,
        requires_explicit_request: true,
        supports_apply: true,
        selector: select_admitted_private_image,
    },
];

struct FinalOutputSelectionContext<'a> {
    plan: &'a nuisc::linker::LinkPlan,
    registration: &'static FinalOutputSelectionPolicyRegistration,
    registry_hash: &'a str,
    explicit_request: bool,
    apply: bool,
}

struct SelectionReportParts<'a> {
    status: &'a str,
    selection_ready: bool,
    installation_attempted: bool,
    selected: bool,
    provider_id: Option<String>,
    admission_status: &'a str,
    admission_contract: Option<String>,
    admission_receipt_file: Option<String>,
    admission_receipt_valid: Option<bool>,
    admission_receipt_hash_sha256: Option<String>,
    admission_verification_ledger_sha256: Option<String>,
    candidate_image_span_bytes: Option<usize>,
    candidate_image_sha256: Option<String>,
    selected_output_path: &'a Path,
    selected_output: OutputSnapshot,
    selected_output_identity_matches: bool,
    publication_contract: Option<String>,
    publication_status: &'a str,
    publication_ledger_sha256: Option<String>,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalOutputSelectionRegistryValidation {
    pub(crate) contract: &'static str,
    pub(crate) registry_hash: String,
    pub(crate) registration_count: usize,
    pub(crate) default_policy_id: Option<&'static str>,
    pub(crate) valid: bool,
    pub(crate) issues: Vec<String>,
}

pub(crate) fn final_output_selection_registry_validation() -> FinalOutputSelectionRegistryValidation
{
    let mut ids = BTreeSet::new();
    let mut defaults = Vec::new();
    let mut issues = Vec::new();
    for registration in REGISTERED_SELECTION_POLICIES {
        if registration.policy_id.is_empty() {
            issues.push("final-output selection policy id is empty".to_owned());
        } else if !ids.insert(registration.policy_id) {
            issues.push(format!(
                "duplicate final-output selection policy `{}`",
                registration.policy_id
            ));
        }
        if registration.default_policy {
            defaults.push(registration.policy_id);
        }
        if registration.policy_status != "ready" {
            issues.push(format!(
                "final-output selection policy `{}` has unsupported status `{}`",
                registration.policy_id, registration.policy_status
            ));
        }
        if registration.requires_explicit_request && registration.default_policy {
            issues.push(format!(
                "final-output selection policy `{}` cannot be both default and explicit-only",
                registration.policy_id
            ));
        }
    }
    if defaults.len() != 1 {
        issues.push(format!(
            "final-output selection registry requires exactly one default policy, found {}",
            defaults.len()
        ));
    }
    FinalOutputSelectionRegistryValidation {
        contract: FINAL_OUTPUT_SELECTION_REGISTRY_CONTRACT,
        registry_hash: final_output_selection_registry_hash(),
        registration_count: REGISTERED_SELECTION_POLICIES.len(),
        default_policy_id: defaults.first().copied(),
        valid: issues.is_empty(),
        issues,
    }
}

pub(crate) fn evaluate_final_output_selection(
    plan: &nuisc::linker::LinkPlan,
    requested_policy: Option<&str>,
    apply: bool,
) -> Result<NsldFinalOutputSelectionReport, String> {
    let validation = final_output_selection_registry_validation();
    if !validation.valid {
        return Err(format!(
            "final-output selection registry is invalid: {}",
            validation.issues.join("; ")
        ));
    }
    let explicit_request = requested_policy.is_some();
    let policy_id = requested_policy
        .or(validation.default_policy_id)
        .ok_or_else(|| "final-output selection registry has no default policy".to_owned())?;
    let registration = REGISTERED_SELECTION_POLICIES
        .iter()
        .find(|registration| registration.policy_id == policy_id)
        .ok_or_else(|| format!("unknown final-output selection policy `{policy_id}`"))?;
    if registration.requires_explicit_request && !explicit_request {
        return Err(format!(
            "final-output selection policy `{policy_id}` requires an explicit request"
        ));
    }
    if apply && !explicit_request {
        return Err("final-output selection apply requires an explicit policy".to_owned());
    }
    if apply && !registration.supports_apply {
        return Err(format!(
            "final-output selection policy `{policy_id}` does not support apply"
        ));
    }
    let mut report = (registration.selector)(&FinalOutputSelectionContext {
        plan,
        registration,
        registry_hash: &validation.registry_hash,
        explicit_request,
        apply,
    })?;
    report.selection_ledger_sha256 = selection_ledger_sha256(&report);
    Ok(report)
}

pub(crate) fn default_final_output_selection(
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalOutputSelectionReport {
    evaluate_final_output_selection(plan, None, false).unwrap_or_else(|error| {
        blocked_selection_report(
            plan,
            COMPATIBILITY_OUTPUT_POLICY,
            "compatibility-selection-evaluation-failed",
            error,
        )
    })
}

pub(crate) fn validate_final_output_selection_report(
    report: &NsldFinalOutputSelectionReport,
) -> Result<(), String> {
    let validation = final_output_selection_registry_validation();
    if !validation.valid {
        return Err("final-output selection evidence refuses an invalid registry".to_owned());
    }
    if report.contract != FINAL_OUTPUT_SELECTION_EVIDENCE_CONTRACT
        || report.registry_contract != FINAL_OUTPUT_SELECTION_REGISTRY_CONTRACT
        || report.registry_hash != validation.registry_hash
    {
        return Err("final-output selection evidence contract or registry drift".to_owned());
    }
    let Some(registration) = REGISTERED_SELECTION_POLICIES
        .iter()
        .find(|registration| registration.policy_id == report.policy_id)
    else {
        return Err("final-output selection evidence references an unknown policy".to_owned());
    };
    if report.policy_status != registration.policy_status
        || report.selection_kind != registration.selection_kind
        || report.default_policy != registration.default_policy
        || report.issue_count != report.issues.len()
    {
        return Err("final-output selection evidence policy or issue drift".to_owned());
    }
    if report.selection_ledger_sha256.len() != 64
        || !report
            .selection_ledger_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || report.selection_ledger_sha256 != selection_ledger_sha256(report)
    {
        return Err("final-output selection evidence ledger drift".to_owned());
    }
    Ok(())
}

fn preserve_compatibility_output(
    context: &FinalOutputSelectionContext<'_>,
) -> Result<NsldFinalOutputSelectionReport, String> {
    let path = Path::new(&context.plan.final_stage.output_path);
    let snapshot = output_snapshot(path)?;
    let issues = if snapshot.present {
        Vec::new()
    } else {
        vec!["compatibility-output-not-present".to_owned()]
    };
    let ready = snapshot.present && snapshot.sha256.is_some();
    Ok(build_report(
        context,
        SelectionReportParts {
            status: if ready {
                "compatibility-output-preserved"
            } else {
                "compatibility-output-unavailable"
            },
            selection_ready: ready,
            installation_attempted: false,
            selected: ready,
            provider_id: None,
            admission_status: "not-applicable",
            admission_contract: None,
            admission_receipt_file: None,
            admission_receipt_valid: None,
            admission_receipt_hash_sha256: None,
            admission_verification_ledger_sha256: None,
            candidate_image_span_bytes: snapshot.span_bytes,
            candidate_image_sha256: snapshot.sha256.clone(),
            selected_output_path: path,
            selected_output: snapshot,
            selected_output_identity_matches: ready,
            publication_contract: None,
            publication_status: "not-applicable",
            publication_ledger_sha256: None,
            issues,
        },
    ))
}

fn select_admitted_private_image(
    context: &FinalOutputSelectionContext<'_>,
) -> Result<NsldFinalOutputSelectionReport, String> {
    let publication = invoke_registered_private_image_publication(context.plan, context.apply)?;
    Ok(private_image_selection_report(context, publication))
}

fn private_image_selection_report(
    context: &FinalOutputSelectionContext<'_>,
    publication: NsldPrivateImagePublicationReport,
) -> NsldFinalOutputSelectionReport {
    let status = if publication.installed {
        "private-image-selected"
    } else if context.apply {
        "blocked-private-image-selection"
    } else if publication.publication_ready {
        "ready-private-image-selection-plan"
    } else {
        "blocked-private-image-selection-plan"
    };
    let snapshot = OutputSnapshot {
        present: publication.output_present_after,
        span_bytes: publication.output_span_bytes_after,
        sha256: publication.output_sha256_after.clone(),
        executable: publication.output_executable,
    };
    let target_key = publication.target_key.clone();
    let capability_id = publication.capability_id.clone();
    let mut report = build_report(
        context,
        SelectionReportParts {
            status,
            selection_ready: publication.publication_ready,
            installation_attempted: publication.installation_attempted,
            selected: publication.installed,
            provider_id: Some(publication.provider_id),
            admission_status: &publication.admission_status,
            admission_contract: Some(publication.admission_contract),
            admission_receipt_file: Some(publication.admission_receipt_file),
            admission_receipt_valid: Some(publication.admission_receipt_valid),
            admission_receipt_hash_sha256: publication.admission_receipt_hash_sha256,
            admission_verification_ledger_sha256: Some(
                publication.admission_verification_ledger_sha256,
            ),
            candidate_image_span_bytes: Some(publication.source_image_span_bytes),
            candidate_image_sha256: Some(publication.source_image_sha256),
            selected_output_path: Path::new(&publication.output_path),
            selected_output: snapshot,
            selected_output_identity_matches: publication.output_matches_private_image,
            publication_contract: Some(publication.contract),
            publication_status: &publication.status,
            publication_ledger_sha256: Some(publication.publication_ledger_sha256),
            issues: publication.issues,
        },
    );
    report.target_key = Some(target_key);
    report.capability_id = Some(capability_id);
    report
}

fn build_report(
    context: &FinalOutputSelectionContext<'_>,
    parts: SelectionReportParts<'_>,
) -> NsldFinalOutputSelectionReport {
    NsldFinalOutputSelectionReport {
        contract: FINAL_OUTPUT_SELECTION_EVIDENCE_CONTRACT.to_owned(),
        registry_contract: FINAL_OUTPUT_SELECTION_REGISTRY_CONTRACT.to_owned(),
        registry_hash: context.registry_hash.to_owned(),
        policy_id: context.registration.policy_id.to_owned(),
        policy_status: context.registration.policy_status.to_owned(),
        selection_kind: context.registration.selection_kind.to_owned(),
        default_policy: context.registration.default_policy,
        explicit_request: context.explicit_request,
        apply_requested: context.apply,
        status: parts.status.to_owned(),
        selection_ready: parts.selection_ready,
        installation_attempted: parts.installation_attempted,
        selected: parts.selected,
        provider_id: parts.provider_id,
        target_key: None,
        capability_id: None,
        admission_contract: parts.admission_contract,
        admission_status: parts.admission_status.to_owned(),
        admission_receipt_file: parts.admission_receipt_file,
        admission_receipt_valid: parts.admission_receipt_valid,
        admission_receipt_hash_sha256: parts.admission_receipt_hash_sha256,
        admission_verification_ledger_sha256: parts.admission_verification_ledger_sha256,
        candidate_image_span_bytes: parts.candidate_image_span_bytes,
        candidate_image_sha256: parts.candidate_image_sha256,
        selected_output_path: parts.selected_output_path.display().to_string(),
        selected_output_name: output_file_name(parts.selected_output_path),
        selected_output_present: parts.selected_output.present,
        selected_output_span_bytes: parts.selected_output.span_bytes,
        selected_output_sha256: parts.selected_output.sha256,
        selected_output_executable: parts.selected_output.executable,
        selected_output_identity_matches: parts.selected_output_identity_matches,
        publication_contract: parts.publication_contract,
        publication_status: parts.publication_status.to_owned(),
        publication_ledger_sha256: parts.publication_ledger_sha256,
        issue_count: parts.issues.len(),
        issues: parts.issues,
        selection_ledger_sha256: String::new(),
    }
}

fn blocked_selection_report(
    plan: &nuisc::linker::LinkPlan,
    policy_id: &str,
    status: &str,
    error: String,
) -> NsldFinalOutputSelectionReport {
    let mut report = NsldFinalOutputSelectionReport {
        contract: FINAL_OUTPUT_SELECTION_EVIDENCE_CONTRACT.to_owned(),
        registry_contract: FINAL_OUTPUT_SELECTION_REGISTRY_CONTRACT.to_owned(),
        registry_hash: final_output_selection_registry_hash(),
        policy_id: policy_id.to_owned(),
        policy_status: "ready".to_owned(),
        selection_kind: "unavailable".to_owned(),
        default_policy: policy_id == COMPATIBILITY_OUTPUT_POLICY,
        explicit_request: false,
        apply_requested: false,
        status: status.to_owned(),
        selection_ready: false,
        installation_attempted: false,
        selected: false,
        provider_id: None,
        target_key: None,
        capability_id: None,
        admission_contract: None,
        admission_status: "not-applicable".to_owned(),
        admission_receipt_file: None,
        admission_receipt_valid: None,
        admission_receipt_hash_sha256: None,
        admission_verification_ledger_sha256: None,
        candidate_image_span_bytes: None,
        candidate_image_sha256: None,
        selected_output_path: plan.final_stage.output_path.clone(),
        selected_output_name: output_file_name(Path::new(&plan.final_stage.output_path)),
        selected_output_present: false,
        selected_output_span_bytes: None,
        selected_output_sha256: None,
        selected_output_executable: false,
        selected_output_identity_matches: false,
        publication_contract: None,
        publication_status: "not-applicable".to_owned(),
        publication_ledger_sha256: None,
        issue_count: 1,
        issues: vec![error],
        selection_ledger_sha256: String::new(),
    };
    report.selection_ledger_sha256 = selection_ledger_sha256(&report);
    report
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
                "failed to inspect final-output selection path `{}`: {error}",
                path.display()
            ));
        }
    };
    if !metadata.is_file() {
        return Err(format!(
            "final-output selection path `{}` is not a regular file",
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

fn sha256_file(path: &Path) -> Result<(usize, String), String> {
    let mut file = File::open(path).map_err(|error| {
        format!(
            "failed to read final-output selection path `{}`: {error}",
            path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    let mut span_bytes = 0usize;
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut chunk).map_err(|error| {
            format!(
                "failed to hash final-output selection path `{}`: {error}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        span_bytes = span_bytes
            .checked_add(read)
            .ok_or_else(|| "final-output selection size overflow".to_owned())?;
        hasher.update(&chunk[..read]);
    }
    Ok((span_bytes, format!("{:x}", hasher.finalize())))
}

#[cfg(unix)]
fn output_is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn output_is_executable(_metadata: &fs::Metadata) -> bool {
    true
}

fn final_output_selection_registry_hash() -> String {
    let mut registrations = REGISTERED_SELECTION_POLICIES.iter().collect::<Vec<_>>();
    registrations.sort_by_key(|registration| registration.policy_id);
    let mut material = format!("contract={FINAL_OUTPUT_SELECTION_REGISTRY_CONTRACT}\n");
    for registration in registrations {
        material.push_str(&format!(
            "policy={}\nstatus={}\nkind={}\ndefault={}\nexplicit={}\napply={}\n",
            registration.policy_id,
            registration.policy_status,
            registration.selection_kind,
            registration.default_policy,
            registration.requires_explicit_request,
            registration.supports_apply
        ));
    }
    sha256_hex(material.as_bytes())
}

fn output_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_owned()
}

fn selection_ledger_sha256(report: &NsldFinalOutputSelectionReport) -> String {
    let mut material = String::new();
    for value in [
        report.contract.as_str(),
        report.registry_contract.as_str(),
        report.registry_hash.as_str(),
        report.policy_id.as_str(),
        report.policy_status.as_str(),
        report.selection_kind.as_str(),
        report.status.as_str(),
        report.provider_id.as_deref().unwrap_or("none"),
        report.target_key.as_deref().unwrap_or("none"),
        report.capability_id.as_deref().unwrap_or("none"),
        report.admission_contract.as_deref().unwrap_or("none"),
        report.admission_status.as_str(),
        report.admission_receipt_file.as_deref().unwrap_or("none"),
        report
            .admission_receipt_hash_sha256
            .as_deref()
            .unwrap_or("none"),
        report
            .admission_verification_ledger_sha256
            .as_deref()
            .unwrap_or("none"),
        report
            .admission_receipt_valid
            .map(|value| if value { "true" } else { "false" })
            .unwrap_or("none"),
        report.candidate_image_sha256.as_deref().unwrap_or("none"),
        report.selected_output_name.as_str(),
        report.selected_output_sha256.as_deref().unwrap_or("none"),
        report.publication_contract.as_deref().unwrap_or("none"),
        report.publication_status.as_str(),
        report
            .publication_ledger_sha256
            .as_deref()
            .unwrap_or("none"),
    ] {
        material.push_str(&format!("text:{}:{value}\n", value.len()));
    }
    material.push_str(&format!(
        "flags={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
        report.default_policy,
        report.explicit_request,
        report.apply_requested,
        report.selection_ready,
        report.installation_attempted,
        report.selected,
        report.admission_receipt_valid.unwrap_or(false),
        report.candidate_image_span_bytes.unwrap_or(0),
        report.selected_output_present,
        report.selected_output_span_bytes.unwrap_or(0),
        report.selected_output_executable,
        report.selected_output_identity_matches
    ));
    for issue in &report.issues {
        material.push_str(&format!("issue:{}:{issue}\n", issue.len()));
    }
    sha256_hex(material.as_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "final_executable_output_selection_tests.rs"]
mod tests;
