use super::*;

pub(super) fn single_source_workflow_next_step_label() -> &'static str {
    "check"
}

pub(super) fn recommended_single_source_workflow_command() -> &'static str {
    "nuis check <input.ns>"
}

pub(crate) struct WorkflowRecommendation {
    pub(crate) label: &'static str,
    pub(crate) command: &'static str,
    pub(crate) reason: &'static str,
}

pub(crate) struct WorkflowSourceProfile {
    pub(crate) source_kind: &'static str,
    pub(crate) workflow_kind: &'static str,
    pub(crate) workflow_brief: &'static str,
    pub(crate) workflow_samples: &'static str,
}

pub(crate) struct WorkflowFrontdoorSurface {
    pub(crate) source_kind: &'static str,
    pub(crate) workflow_kind: &'static str,
    pub(crate) workflow_brief: &'static str,
    pub(crate) workflow_samples: &'static str,
    pub(crate) recommended_next_step: &'static str,
    pub(crate) recommended_command: &'static str,
    pub(crate) recommended_reason: &'static str,
}

pub(crate) const FRONTDOOR_READING_ORDER: &str =
    "closure_summary -> dev_tensor_weakest_task_card_handoff";
pub(crate) const FRONTDOOR_SAMPLE_CLOSURE_SUMMARY: &str =
    "closure_summary_status -> closure_summary_next_action -> closure_summary_next_command";
pub(crate) const FRONTDOOR_SAMPLE_TENSOR_HANDOFF: &str =
    "dev_tensor_weakest_task_card_coordinate -> dev_tensor_weakest_task_card_handoff_coordinate -> dev_tensor_weakest_task_card_handoff_command";

pub(crate) fn build_workflow_frontdoor_surface(
    profile: WorkflowSourceProfile,
    recommendation: WorkflowRecommendation,
) -> WorkflowFrontdoorSurface {
    WorkflowFrontdoorSurface {
        source_kind: profile.source_kind,
        workflow_kind: profile.workflow_kind,
        workflow_brief: profile.workflow_brief,
        workflow_samples: profile.workflow_samples,
        recommended_next_step: recommendation.label,
        recommended_command: recommendation.command,
        recommended_reason: recommendation.reason,
    }
}

#[allow(dead_code)]
pub(crate) fn workflow_frontdoor_json_fields(surface: &WorkflowFrontdoorSurface) -> Vec<String> {
    vec![
        json_field("source_kind", surface.source_kind),
        json_field("workflow_kind", surface.workflow_kind),
        json_field("workflow_brief", surface.workflow_brief),
        json_field("workflow_samples", surface.workflow_samples),
        json_field("recommended_next_step", surface.recommended_next_step),
        json_field("recommended_command", surface.recommended_command),
        json_field("recommended_reason", surface.recommended_reason),
    ]
}

pub(crate) fn append_workflow_frontdoor_json_fields(
    out: &mut String,
    surface: &WorkflowFrontdoorSurface,
) {
    append_json_field_strings(
        out,
        vec![
            json_field("source_kind", surface.source_kind),
            json_field("workflow_kind", surface.workflow_kind),
            json_field("workflow_brief", surface.workflow_brief),
            json_field("workflow_samples", surface.workflow_samples),
            json_field("recommended_next_step", surface.recommended_next_step),
            json_field("recommended_command", surface.recommended_command),
            json_field("recommended_reason", surface.recommended_reason),
        ],
    );
}

pub(crate) fn workflow_frontdoor_json_object_field(surface: &WorkflowFrontdoorSurface) -> String {
    let mut out = String::from("\"frontdoor\":{");
    append_workflow_frontdoor_json_fields(&mut out, surface);
    out.push('}');
    out
}

pub(crate) fn print_workflow_frontdoor_surface(surface: &WorkflowFrontdoorSurface) {
    println!("  frontdoor.source_kind: {}", surface.source_kind);
    println!("  frontdoor.workflow_kind: {}", surface.workflow_kind);
    println!("  frontdoor.workflow_brief: {}", surface.workflow_brief);
    print_scheduler_sample_field("frontdoor.workflow_samples", surface.workflow_samples);
    println!(
        "  frontdoor.recommended_next_step: {}",
        surface.recommended_next_step
    );
    println!(
        "  frontdoor.recommended_command: {}",
        surface.recommended_command
    );
    println!(
        "  frontdoor.recommended_reason: {}",
        surface.recommended_reason
    );
}

pub(crate) fn single_source_workflow_source_profile() -> WorkflowSourceProfile {
    WorkflowSourceProfile {
        source_kind: "single-file",
        workflow_kind: "compile_workflow",
        workflow_brief: single_source_compile_workflow_brief(),
        workflow_samples: single_source_compile_samples_brief(),
    }
}

pub(crate) fn project_compile_workflow_source_profile() -> WorkflowSourceProfile {
    WorkflowSourceProfile {
        source_kind: "project",
        workflow_kind: "project_compile_workflow",
        workflow_brief: nuisc::project_compile_workflow_brief(),
        workflow_samples: nuisc::project_compile_samples_brief(),
    }
}

pub(crate) fn project_frontdoor_surface(
    plan: &nuisc::project::ProjectCompilationPlan,
    declared_tests: &[PathBuf],
    missing_tests: &[PathBuf],
    galaxy_doctor: &galaxy::GalaxyDoctorReport,
    galaxy_check_invalid: bool,
    has_hidden_manual_only_library_modules: bool,
) -> WorkflowFrontdoorSurface {
    let recommendation = recommend_project_workflow_step(
        plan,
        declared_tests,
        missing_tests,
        galaxy_doctor,
        galaxy_check_invalid,
        has_hidden_manual_only_library_modules,
    );
    build_workflow_frontdoor_surface(project_compile_workflow_source_profile(), recommendation)
}

pub(crate) fn single_source_frontdoor_surface() -> WorkflowFrontdoorSurface {
    build_workflow_frontdoor_surface(
        single_source_workflow_source_profile(),
        WorkflowRecommendation {
            label: single_source_workflow_next_step_label(),
            command: recommended_single_source_workflow_command(),
            reason: "single-file inputs usually want direct compile truth first, so `check` stays the best default front-door step",
        },
    )
}

pub(crate) fn toolchain_frontdoor_surface() -> WorkflowFrontdoorSurface {
    build_workflow_frontdoor_surface(
        WorkflowSourceProfile {
            source_kind: "toolchain",
            workflow_kind: "default_compile_frontdoor",
            workflow_brief: "workflow -> project_doctor -> check -> test -> build -> artifact_doctor -> nsld_drive -> run_artifact -> release_check",
            workflow_samples: "workflow=nuis workflow [input]; doctor=nuis project-doctor [project-dir|nuis.toml]; check=nuis check [input]; test=nuis test [input]; build=nuis build [input] <output-dir>; artifact=nuis artifact-doctor <output-dir>; linker=nsld drive <output-dir>/nuis.build.manifest.toml --apply; run=nuis run-artifact <output-dir>; release=nuis release-check [input] [output-dir]",
        },
        WorkflowRecommendation {
            label: "workflow",
            command: "nuis workflow [--json] [input.ns|project-dir|nuis.toml]",
            reason: "the compile frontdoor should classify the input shape first, then route into the right project or single-file workflow branch",
        },
    )
}

pub(crate) fn recommend_project_workflow_step(
    plan: &nuisc::project::ProjectCompilationPlan,
    declared_tests: &[PathBuf],
    missing_tests: &[PathBuf],
    galaxy_doctor: &galaxy::GalaxyDoctorReport,
    galaxy_check_invalid: bool,
    has_hidden_manual_only_library_modules: bool,
) -> WorkflowRecommendation {
    let deps_len = galaxy_doctor.dependencies.len();
    let any_lock_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.locked);
    let any_install_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.installed);
    if galaxy_check_invalid {
        return WorkflowRecommendation {
            label: "galaxy_check",
            command: "nuis galaxy check <project-dir|nuis.toml>",
            reason: "project packaging metadata is currently invalid, so the next step should re-check and fix the galaxy-side project contract first",
        };
    }
    match galaxy_doctor.lock_status.as_str() {
        "missing" if deps_len > 0 => {
            return WorkflowRecommendation {
                label: "galaxy_lock_deps",
                command: "nuis galaxy lock-deps <project-dir|nuis.toml>",
                reason: "the project already declares galaxy dependencies but does not yet have a lockfile",
            };
        }
        "invalid" => {
            return WorkflowRecommendation {
                label: "galaxy_verify_lock",
                command: "nuis galaxy verify-lock <project-dir|nuis.toml>",
                reason: "the current galaxy lockfile is invalid and should be repaired or regenerated before deeper compile work",
            };
        }
        _ => {}
    }
    if any_lock_missing && deps_len > 0 && galaxy_doctor.lock_status == "ok" {
        return WorkflowRecommendation {
            label: "galaxy_lock_refresh",
            command: "nuis galaxy lock-deps <project-dir|nuis.toml>",
            reason: "the lockfile exists, but some declared galaxy dependencies are not represented in it yet",
        };
    }
    if any_install_missing && galaxy_doctor.lock_status == "ok" {
        return WorkflowRecommendation {
            label: "galaxy_sync_deps",
            command: "nuis galaxy sync-deps <project-dir|nuis.toml>",
            reason: "the dependency lock is valid, but some locked galaxy packages are not materialized locally yet",
        };
    }
    if has_hidden_manual_only_library_modules {
        return WorkflowRecommendation {
            label: "project_imports_apply_suggested",
            command: "nuis project-imports --apply-suggested <project-dir|nuis.toml>",
            reason: "the project still has manual-only galaxy library modules hidden from project scope, so the highest-value next step is to write the suggested galaxy_imports entries first",
        };
    }
    if !plan.abi_resolution.explicit {
        return WorkflowRecommendation {
            label: "project_lock_abi",
            command: "nuis project-lock-abi <project-dir|nuis.toml>",
            reason: "the project is still using auto-recommended ABI selection, so freezing the current ABI choice is the highest-value stabilizing step",
        };
    }
    if !missing_tests.is_empty() {
        return WorkflowRecommendation {
            label: "project_status",
            command: "nuis project-status <project-dir|nuis.toml>",
            reason: "some declared project tests are missing on disk, so the next step should inspect and fix the declared test surface",
        };
    }
    if declared_tests.is_empty() {
        return WorkflowRecommendation {
            label: "test",
            command: "nuis test <project-dir|nuis.toml>",
            reason: "the project has no explicit declared tests yet, so the next useful step is to run the current language-level test sweep and then decide whether to add dedicated project tests",
        };
    }
    WorkflowRecommendation {
        label: "check",
        command: "nuis check <project-dir|nuis.toml>",
        reason: "the obvious project-shape blockers are already under control, so the next step is to re-check compile truth directly",
    }
}
