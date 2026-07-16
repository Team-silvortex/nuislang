use crate::{
    artifact_doctor_mirrors::{
        collect_backend_artifact_payload_evidence, collect_payload_decoder_manifest_mirror,
    },
    resolve_frontdoor_build_manifest_path,
    run_artifact::{run_artifact_prelaunch_summary, self_contained_link_plan_selected},
    workflow::{
        load_link_plan_for_output_dir, nsld_drive_apply_until_clean_command_for_output_dir,
    },
};
use std::path::{Path, PathBuf};

pub(crate) use crate::artifact_doctor_mirrors::{
    BackendArtifactPayloadEvidence, PayloadDecoderManifestMirror,
};

pub(crate) fn run_build_output_self_check(
    output_dir: &Path,
) -> Result<ArtifactDoctorReport, String> {
    let manifest_path = resolve_frontdoor_build_manifest_path(output_dir)?;
    let doctor = probe_artifact_doctor(output_dir);
    let manifest_report = nuisc::aot::verify_build_manifest(&manifest_path).map_err(|error| {
        format!(
            "build self-check could not verify manifest `{}`: {error}; next step: {} ({})",
            manifest_path.display(),
            doctor.recommended_next_step,
            doctor.recommended_command
        )
    })?;
    nuisc::aot::verify_nuis_compiled_artifact(Path::new(&manifest_report.artifact_path)).map_err(
        |error| {
            format!(
                "build self-check could not verify artifact `{}`: {error}; next step: {} ({})",
                manifest_report.artifact_path,
                doctor.recommended_next_step,
                doctor.recommended_command
            )
        },
    )?;
    let self_contained_route = doctor
        .output_dir
        .as_deref()
        .is_some_and(self_contained_link_plan_selected);
    if !doctor.ready_to_run && !self_contained_route {
        return Err(format!(
            "build self-check found incomplete runnable output in `{}`; next step: {} ({})",
            output_dir.display(),
            doctor.recommended_next_step,
            doctor.recommended_command
        ));
    }
    Ok(doctor)
}

pub(crate) fn collect_artifact_output_diagnostics(
    input: &Path,
    report: &ArtifactDoctorReport,
) -> ArtifactOutputDiagnostics {
    let (self_check_ready, self_check_error) =
        build_output_self_check_status(report.output_dir.as_deref());
    let project_snapshot = collect_project_validation_snapshot(input, Some(report));
    ArtifactOutputDiagnostics {
        artifact_diagnostic_code: artifact_diagnostic_code(report),
        self_check: SelfCheckSummary {
            ready: self_check_ready,
            code: self_check_code(report.output_dir.as_deref(), self_check_error.as_deref()),
            error: self_check_error,
        },
        project_checks: ProjectCheckSummary {
            code: project_checks_code(project_snapshot.as_ref()),
            snapshot: project_snapshot,
        },
        link_plan: LinkPlanSummary {
            plan: report
                .output_dir
                .as_ref()
                .and_then(|output_dir| load_link_plan_for_output_dir(output_dir)),
        },
        backend_artifact_payload_evidence: collect_backend_artifact_payload_evidence(
            report.output_dir.as_deref(),
        ),
    }
}

fn build_output_self_check_status(output_dir: Option<&Path>) -> (bool, Option<String>) {
    let Some(output_dir) = output_dir else {
        return (
            false,
            Some("no output_dir available for self-check".to_owned()),
        );
    };
    if !output_dir.exists() {
        return (
            false,
            Some(format!(
                "`{}` does not contain `nuis.build.manifest.toml`",
                output_dir.display()
            )),
        );
    }
    match run_build_output_self_check(output_dir) {
        Ok(_) => (true, None),
        Err(error) => (false, Some(error)),
    }
}

fn artifact_diagnostic_code(report: &ArtifactDoctorReport) -> &'static str {
    if !report.manifest_exists && !report.artifact_exists && !report.binary_exists {
        "missing_outputs"
    } else if report.manifest_exists && !report.manifest_verified {
        "manifest_invalid"
    } else if report.artifact_exists && !report.artifact_verified {
        "artifact_invalid"
    } else if report.ready_to_run {
        "ready_to_run"
    } else if report.manifest_exists || report.artifact_exists {
        "partial_outputs"
    } else {
        "binary_only"
    }
}

fn self_check_code(output_dir: Option<&Path>, self_check_error: Option<&str>) -> &'static str {
    match self_check_error {
        None => "ok",
        Some(_) if output_dir.is_none() => "no_output_dir",
        Some(error) if error.contains("does not contain `nuis.build.manifest.toml`") => {
            "missing_build_manifest"
        }
        Some(error)
            if error.contains("expected an output directory or `nuis.build.manifest.toml`") =>
        {
            "invalid_artifact_input"
        }
        Some(error) if error.contains("could not verify manifest") => "manifest_verify_failed",
        Some(error) if error.contains("could not verify artifact") => "artifact_verify_failed",
        Some(error) if error.contains("incomplete runnable output") => "incomplete_runnable_output",
        Some(_) => "self_check_failed",
    }
}

fn project_checks_code(snapshot: Option<&ProjectValidationSnapshot>) -> &'static str {
    let Some(snapshot) = snapshot else {
        return "unavailable";
    };
    if snapshot.abi_checks.iter().any(|check| !check.ok) {
        "abi_checks_failed"
    } else if snapshot.registry_checks.iter().any(|check| !check.ok) {
        "registry_checks_failed"
    } else if snapshot.lowering_checks.iter().any(|check| !check.ok) {
        "lowering_checks_failed"
    } else {
        "ok"
    }
}

fn collect_project_validation_snapshot(
    input: &Path,
    doctor: Option<&ArtifactDoctorReport>,
) -> Option<ProjectValidationSnapshot> {
    let mut candidates = vec![input.to_path_buf()];
    if let Some(manifest_path) = doctor
        .and_then(|report| report.manifest_path.clone())
        .or_else(|| resolve_frontdoor_build_manifest_path(input).ok())
    {
        if let Ok(manifest_report) = nuisc::aot::verify_build_manifest(&manifest_path) {
            let source_input = PathBuf::from(&manifest_report.input);
            candidates.push(source_input.clone());
            if let Some(parent) = source_input.parent() {
                candidates.push(parent.to_path_buf());
            }
        }
    }
    for candidate in candidates {
        let Ok(project) = nuisc::project::load_project(&candidate) else {
            continue;
        };
        let Ok(plan) = nuisc::project::build_project_compilation_plan(&project) else {
            continue;
        };
        let Ok(abi_checks) =
            nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)
        else {
            continue;
        };
        let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
        let lowering_checks =
            nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
        return Some(ProjectValidationSnapshot {
            project_root: project.root.clone(),
            abi_checks,
            registry_checks,
            lowering_checks,
        });
    }
    None
}

pub(crate) fn probe_artifact_doctor(input: &Path) -> ArtifactDoctorReport {
    let mut source_kind = "binary".to_owned();
    let mut output_dir = if input.is_dir() {
        Some(input.to_path_buf())
    } else if looks_like_artifact_output_dir(input) {
        Some(input.to_path_buf())
    } else {
        input.parent().map(Path::to_path_buf)
    };
    let mut manifest_path = None;
    let mut artifact_path = None;
    let mut binary_path = None;

    if input.is_dir() || looks_like_artifact_output_dir(input) {
        source_kind = "output_dir".to_owned();
        let candidate_manifest = input.join("nuis.build.manifest.toml");
        let candidate_artifact = input.join("nuis.compiled.artifact");
        if candidate_manifest.exists() {
            manifest_path = Some(candidate_manifest);
        }
        if candidate_artifact.exists() {
            artifact_path = Some(candidate_artifact);
        }
    } else if input.file_name().and_then(|value| value.to_str()) == Some("nuis.build.manifest.toml")
    {
        source_kind = "manifest".to_owned();
        manifest_path = Some(input.to_path_buf());
        output_dir = input.parent().map(Path::to_path_buf);
    } else if input.file_name().and_then(|value| value.to_str()) == Some("nuis.compiled.artifact") {
        source_kind = "artifact".to_owned();
        artifact_path = Some(input.to_path_buf());
        output_dir = input.parent().map(Path::to_path_buf);
    } else {
        binary_path = Some(input.to_path_buf());
        output_dir = input.parent().map(Path::to_path_buf);
        if let Some(dir) = output_dir.as_ref() {
            let candidate_manifest = dir.join("nuis.build.manifest.toml");
            let candidate_artifact = dir.join("nuis.compiled.artifact");
            if candidate_manifest.exists() {
                manifest_path = Some(candidate_manifest);
            }
            if candidate_artifact.exists() {
                artifact_path = Some(candidate_artifact);
            }
        }
    }

    let mut manifest_verified = false;
    let mut artifact_verified = false;
    let mut manifest_verify_error = None;
    let mut artifact_verify_error = None;
    let mut artifact_container_kind = None;
    let mut artifact_container_version = None;
    let mut artifact_section_count = None;
    let mut artifact_section_names = Vec::new();
    let mut artifact_section_table_valid = None;
    let mut lowering_unit_count = None;
    let mut lowering_domain_families = Vec::new();
    let mut lowering_targets = Vec::new();
    let mut lowering_units = Vec::new();

    if let Some(path) = manifest_path.as_ref() {
        match nuisc::aot::verify_build_manifest(path) {
            Ok(report) => {
                manifest_verified = true;
                artifact_path = Some(PathBuf::from(&report.artifact_path));
                binary_path =
                    Some(Path::new(&report.output_dir).join(&report.artifact_binary_name));
                output_dir = Some(PathBuf::from(&report.output_dir));
            }
            Err(error) => manifest_verify_error = Some(error),
        }
    }

    if let Some(path) = artifact_path.as_ref() {
        match nuisc::aot::inspect_nuis_compiled_artifact_container(path) {
            Ok(container) => {
                artifact_container_kind = Some(container.container_kind);
                artifact_container_version = Some(container.binary_version);
                artifact_section_count = Some(container.section_count);
                artifact_section_names = container.section_names;
                artifact_section_table_valid = Some(container.section_table_valid);
                lowering_unit_count = Some(container.lowering_unit_count);
                lowering_domain_families = container.lowering_domain_families;
                lowering_targets = container.lowering_targets;
                lowering_units = container.lowering_units;
            }
            Err(error) => {
                artifact_verify_error = Some(error);
            }
        }
        match nuisc::aot::verify_nuis_compiled_artifact(path) {
            Ok(report) => {
                artifact_verified = true;
                if binary_path.is_none() {
                    let base = path.parent().unwrap_or_else(|| Path::new("."));
                    binary_path = Some(base.join(report.binary_name));
                }
            }
            Err(error) => {
                artifact_verify_error = Some(error);
                if binary_path.is_none() {
                    if let Ok(artifact) = nuisc::aot::parse_nuis_compiled_artifact(path) {
                        let base = path.parent().unwrap_or_else(|| Path::new("."));
                        binary_path = Some(base.join(artifact.binary_name));
                    }
                }
            }
        }
    }

    let manifest_exists = manifest_path.as_ref().is_some_and(|path| path.exists());
    let artifact_exists = artifact_path.as_ref().is_some_and(|path| path.exists());
    let binary_exists = binary_path.as_ref().is_some_and(|path| path.exists());
    let self_contained_route = output_dir
        .as_deref()
        .is_some_and(self_contained_link_plan_selected);
    let nsld_handoff_ready = output_dir.as_deref().is_some_and(|path| {
        run_artifact_prelaunch_summary(Some(path), None).nsld_runtime_handoff_ready()
    });
    let direct_host_binary_ready = binary_exists && !self_contained_route;
    let ready_to_run =
        (direct_host_binary_ready || nsld_handoff_ready) && manifest_verified && artifact_verified;
    let payload_decoder_manifest = collect_payload_decoder_manifest_mirror(output_dir.as_deref());

    let (recommended_next_step, recommended_command, recommended_reason) = if !manifest_exists
        && !artifact_exists
        && !binary_exists
    {
        (
            "build".to_owned(),
            "nuis build <input> <output-dir>".to_owned(),
            "no recognizable native artifact outputs were found yet, so the next step is to rebuild a fresh output directory".to_owned(),
        )
    } else if manifest_exists && !manifest_verified {
        (
            "verify_build_manifest".to_owned(),
            output_dir
                .as_ref()
                .map(|path| format!("nuis verify-build-manifest {}", path.display()))
                .or_else(|| {
                    manifest_path
                        .as_ref()
                        .map(|path| format!("nuis verify-build-manifest {}", path.display()))
                })
                .unwrap_or_else(|| "nuis verify-build-manifest <output-dir>".to_owned()),
            "the manifest exists but does not currently pass verification, so the next step is to inspect that contract boundary directly".to_owned(),
        )
    } else if artifact_exists && !artifact_verified {
        (
            "verify_artifact".to_owned(),
            format!(
                "nuis verify-artifact {}",
                artifact_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<nuis.compiled.artifact>".to_owned())
            ),
            "the compiled artifact exists but does not currently pass verification, so the next step is to inspect the packaged binary bundle directly".to_owned(),
        )
    } else if ready_to_run {
        (
            "run_artifact".to_owned(),
            output_dir
                .as_ref()
                .map(|path| format!("nuis run-artifact {}", path.display()))
                .or_else(|| {
                    manifest_path
                        .as_ref()
                        .or(artifact_path.as_ref())
                        .or(binary_path.as_ref())
                        .map(|path| format!("nuis run-artifact {}", path.display()))
                })
                .unwrap_or_else(|| "nuis run-artifact <output-dir>".to_owned()),
            "the binary, manifest, and compiled artifact are all present and verified, so the next step is to launch the built output through the nuis frontdoor".to_owned(),
        )
    } else if self_contained_route && manifest_verified && artifact_verified {
        (
            "nsld_drive".to_owned(),
            output_dir
                .as_ref()
                .map(|path| nsld_drive_apply_until_clean_command_for_output_dir(path))
                .unwrap_or_else(|| "nsld drive <output-dir> --apply --until-clean".to_owned()),
            "self-contained Nuis image packaging is selected, so the next step is to materialize the Nsld-owned image and runtime handoff artifacts".to_owned(),
        )
    } else if manifest_exists || artifact_exists {
        (
            "inspect_artifact".to_owned(),
            output_dir
                .as_ref()
                .map(|path| format!("nuis inspect-artifact {}", path.display()))
                .or_else(|| {
                    manifest_path
                        .as_ref()
                        .or(artifact_path.as_ref())
                        .map(|path| format!("nuis inspect-artifact {}", path.display()))
                })
                .unwrap_or_else(|| "nuis inspect-artifact <output-dir>".to_owned()),
            "some artifact outputs are present, but the closure is not fully ready yet, so the next step is to inspect the available bundle metadata".to_owned(),
        )
    } else {
        (
            "run_artifact".to_owned(),
            format!(
                "nuis run-artifact {}",
                binary_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<binary-path>".to_owned())
            ),
            "only the binary path is currently visible, so the next step is to launch it through the nuis artifact runner".to_owned(),
        )
    };

    ArtifactDoctorReport {
        source_kind,
        input: input.to_path_buf(),
        output_dir,
        manifest_path,
        artifact_path,
        binary_path,
        manifest_exists,
        artifact_exists,
        binary_exists,
        manifest_verified,
        artifact_verified,
        ready_to_run,
        recommended_next_step,
        recommended_command,
        recommended_reason,
        manifest_verify_error,
        artifact_verify_error,
        payload_decoder_manifest,
        artifact_container_kind,
        artifact_container_version,
        artifact_section_count,
        artifact_section_names,
        artifact_section_table_valid,
        lowering_unit_count,
        lowering_domain_families,
        lowering_targets,
        lowering_units,
    }
}

fn looks_like_artifact_output_dir(input: &Path) -> bool {
    !input.exists() && input.extension().is_none()
}

pub(crate) struct ArtifactDoctorReport {
    pub(crate) source_kind: String,
    pub(crate) input: PathBuf,
    pub(crate) output_dir: Option<PathBuf>,
    pub(crate) manifest_path: Option<PathBuf>,
    pub(crate) artifact_path: Option<PathBuf>,
    pub(crate) binary_path: Option<PathBuf>,
    pub(crate) manifest_exists: bool,
    pub(crate) artifact_exists: bool,
    pub(crate) binary_exists: bool,
    pub(crate) manifest_verified: bool,
    pub(crate) artifact_verified: bool,
    pub(crate) artifact_container_kind: Option<String>,
    pub(crate) artifact_container_version: Option<u16>,
    pub(crate) artifact_section_count: Option<usize>,
    pub(crate) artifact_section_names: Vec<String>,
    pub(crate) artifact_section_table_valid: Option<bool>,
    pub(crate) lowering_unit_count: Option<usize>,
    pub(crate) lowering_domain_families: Vec<String>,
    pub(crate) lowering_targets: Vec<String>,
    pub(crate) lowering_units: Vec<nuisc::aot::NuisCompiledArtifactLoweringUnitInspect>,
    pub(crate) ready_to_run: bool,
    pub(crate) recommended_next_step: String,
    pub(crate) recommended_command: String,
    pub(crate) recommended_reason: String,
    pub(crate) manifest_verify_error: Option<String>,
    pub(crate) artifact_verify_error: Option<String>,
    pub(crate) payload_decoder_manifest: PayloadDecoderManifestMirror,
}

pub(crate) struct ProjectValidationSnapshot {
    pub(crate) project_root: PathBuf,
    pub(crate) abi_checks: Vec<nuisc::project::ProjectAbiSelectionCheck>,
    pub(crate) registry_checks: Vec<nuisc::registry::ProjectDomainRegistryCheck>,
    pub(crate) lowering_checks: Vec<nuisc::project::ProjectLoweringSelectionView>,
}

pub(crate) struct SelfCheckSummary {
    pub(crate) ready: bool,
    pub(crate) error: Option<String>,
    pub(crate) code: &'static str,
}

pub(crate) struct ProjectCheckSummary {
    pub(crate) snapshot: Option<ProjectValidationSnapshot>,
    pub(crate) code: &'static str,
}

impl ProjectCheckSummary {
    pub(crate) fn available(&self) -> bool {
        self.snapshot.is_some()
    }
}

pub(crate) struct LinkPlanSummary {
    pub(crate) plan: Option<nuisc::linker::LinkPlan>,
}

impl LinkPlanSummary {
    pub(crate) fn as_ref(&self) -> Option<&nuisc::linker::LinkPlan> {
        self.plan.as_ref()
    }
}

pub(crate) struct ArtifactOutputDiagnostics {
    pub(crate) artifact_diagnostic_code: &'static str,
    pub(crate) self_check: SelfCheckSummary,
    pub(crate) project_checks: ProjectCheckSummary,
    pub(crate) link_plan: LinkPlanSummary,
    pub(crate) backend_artifact_payload_evidence: BackendArtifactPayloadEvidence,
}
