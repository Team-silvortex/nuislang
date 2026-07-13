use super::*;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_project_doctor_text_summary(input: &Path) -> Result<Vec<String>, String> {
    let mut out = String::new();
    write_project_doctor_text_summary(&mut out, input)?;
    Ok(out.lines().map(str::to_owned).collect())
}

pub(crate) fn write_project_doctor_text_summary<W: fmt::Write>(
    out: &mut W,
    input: &Path,
) -> Result<(), String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let text_handle_rewrite = nuisc::project::summarize_project_text_handle_rewrites(&project)?;
    let public_surface = crate::public_surface_records(&project);
    let declared_tests = project
        .manifest
        .tests
        .iter()
        .map(|relative| project.root.join(relative))
        .collect::<Vec<_>>();
    let missing_tests = declared_tests
        .iter()
        .filter(|path| !path.exists())
        .cloned()
        .collect::<Vec<_>>();
    let galaxy_manifest_path = project.root.join("galaxy.toml");
    let galaxy_manifest_exists = galaxy_manifest_path.exists();
    let galaxy_check = if galaxy_manifest_exists {
        Some(crate::galaxy::check(&project.root))
    } else {
        None
    };
    let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
    let galaxy_doctor = crate::galaxy::doctor_project(&project.root)?;
    let nova_profile = crate::galaxy::inspect_ns_nova_profile(&project.root)?;
    let nova_stdlib = crate::galaxy::inspect_ns_nova_stdlib(std::path::Path::new("."))?;
    let lock_status = galaxy_doctor.lock_status.clone();
    let lock_error = galaxy_doctor.lock_error.clone();
    let deps_len = galaxy_doctor.dependencies.len();
    let include_galaxy_flow =
        galaxy_manifest_exists || !project.manifest.galaxy_dependencies.is_empty();
    let any_local_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.local_available);
    let any_lock_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.locked);
    let any_install_missing = galaxy_doctor
        .dependencies
        .iter()
        .any(|dependency| !dependency.installed);
    let abi_checks =
        nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)?;
    let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
    let lowering_checks =
        nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
    let hidden_manual_only_library_modules =
        crate::hidden_manual_only_library_modules_for_project(&project);
    let frontdoor = crate::project_frontdoor_surface(
        &plan,
        &declared_tests,
        &missing_tests,
        &galaxy_doctor,
        galaxy_check_invalid,
        !hidden_manual_only_library_modules.is_empty(),
    );
    let artifact_output_dir = crate::default_build_output_dir(input);
    let artifact_report = crate::probe_artifact_doctor(&artifact_output_dir);
    let link_plan = load_link_plan(&artifact_output_dir);
    writeln!(out, "project doctor: {}", project.manifest.name).map_err(|e| e.to_string())?;
    writeln!(out, "  root: {}", project.root.display()).map_err(|e| e.to_string())?;
    writeln!(out, "  manifest: {}", project.manifest_path.display()).map_err(|e| e.to_string())?;
    writeln!(out, "  entry: {}", project.manifest.entry).map_err(|e| e.to_string())?;
    writeln!(out, "  frontdoor.source_kind: {}", frontdoor.source_kind)
        .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  frontdoor.workflow_kind: {}",
        frontdoor.workflow_kind
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  frontdoor.workflow_brief: {}",
        frontdoor.workflow_brief
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  frontdoor.workflow_samples: {}",
        frontdoor.workflow_samples
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  frontdoor.recommended_next_step: {}",
        frontdoor.recommended_next_step
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  frontdoor.recommended_command: {}",
        frontdoor.recommended_command
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  frontdoor.recommended_reason: {}",
        frontdoor.recommended_reason
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  recommended_command: {}",
        frontdoor.recommended_command
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  recommended_reason: {}",
        frontdoor.recommended_reason
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_output_dir: {}",
        artifact_output_dir.display()
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_ready_to_run: {}",
        crate::yes_no(artifact_report.ready_to_run)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_recommended_next_step: {}",
        artifact_report.recommended_next_step
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_recommended_command: {}",
        artifact_report.recommended_command
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_nsld_drive_dry_run_command: {}",
        crate::workflow::nsld_drive_dry_run_command_for_output_dir(&artifact_output_dir)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_nsld_drive_dry_run_json_command: {}",
        crate::workflow::nsld_drive_dry_run_json_command_for_output_dir(&artifact_output_dir)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_nsld_drive_apply_next_command: {}",
        crate::workflow::nsld_drive_apply_next_command_for_output_dir(&artifact_output_dir)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_nsld_drive_apply_next_json_command: {}",
        crate::workflow::nsld_drive_apply_next_json_command_for_output_dir(&artifact_output_dir)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_nsld_drive_apply_until_clean_command: {}",
        crate::workflow::nsld_drive_apply_until_clean_command_for_output_dir(&artifact_output_dir)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_nsld_drive_apply_until_clean_json_command: {}",
        crate::workflow::nsld_drive_apply_until_clean_json_command_for_output_dir(
            &artifact_output_dir
        )
    )
    .map_err(|e| e.to_string())?;
    writeln!(out, "  modules: {}", project.modules.len()).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  text_handle_rewrite_helper_hits: {}",
        text_handle_rewrite.helper_hits
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  text_handle_rewrite_local_hits: {}",
        text_handle_rewrite.local_hits
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  text_handle_rewrite_total_hits: {}",
        text_handle_rewrite.total_hits()
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  public_surface: {}",
        crate::describe_public_surface(&public_surface)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  public_surface_modules: {}",
        crate::describe_public_surface_modules(&public_surface)
    )
    .map_err(|e| e.to_string())?;
    writeln!(out, "  links: {}", project.manifest.links.len()).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_plan: {}",
        nuisc::project::describe_project_compilation_plan(&plan)
    )
    .map_err(|e| e.to_string())?;
    writeln!(out, "  tests_declared: {}", declared_tests.len()).map_err(|e| e.to_string())?;
    writeln!(out, "  tests_missing: {}", missing_tests.len()).map_err(|e| e.to_string())?;
    write_link_plan_text_fields(out, link_plan.as_ref()).map_err(|e| e.to_string())?;
    for path in &declared_tests {
        writeln!(
            out,
            "  test: {} exists={}",
            path.display(),
            crate::yes_no(path.exists())
        )
        .map_err(|e| e.to_string())?;
    }
    writeln!(
        out,
        "  project_compile_workflow: {}",
        nuisc::project_compile_workflow_brief()
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_compile_samples: {}",
        nuisc::project_compile_samples_brief()
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_test_workflow: {}",
        nuisc::project_test_workflow_brief()
    )
    .map_err(|e| e.to_string())?;
    if include_galaxy_flow {
        writeln!(
            out,
            "  project_galaxy_workflow: {}",
            nuisc::project_galaxy_workflow_brief()
        )
        .map_err(|e| e.to_string())?;
    }
    writeln!(
        out,
        "  abi_mode: {}",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    )
    .map_err(|e| e.to_string())?;
    writeln!(out, "  abi_checks: {}", abi_checks.len()).map_err(|e| e.to_string())?;
    writeln!(out, "  registry_checks: {}", registry_checks.len()).map_err(|e| e.to_string())?;
    writeln!(out, "  lowering_checks: {}", lowering_checks.len()).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  galaxy_manifest: {}",
        if galaxy_manifest_exists {
            galaxy_manifest_path.display().to_string()
        } else {
            "<missing>".to_owned()
        }
    )
    .map_err(|e| e.to_string())?;
    match galaxy_check {
        Some(Ok(checked)) => {
            writeln!(out, "  galaxy_check: ok").map_err(|e| e.to_string())?;
            writeln!(
                out,
                "  galaxy_package_kind: {}",
                checked.manifest.package_kind
            )
            .map_err(|e| e.to_string())?;
            writeln!(
                out,
                "  galaxy_framework: {}",
                checked.manifest.framework.as_deref().unwrap_or("<none>")
            )
            .map_err(|e| e.to_string())?;
            writeln!(
                out,
                "  galaxy_include_files: {}",
                checked.include_files.len()
            )
            .map_err(|e| e.to_string())?;
        }
        Some(Err(error)) => {
            writeln!(out, "  galaxy_check: invalid").map_err(|e| e.to_string())?;
            writeln!(out, "  galaxy_error: {}", error).map_err(|e| e.to_string())?;
        }
        None => writeln!(out, "  galaxy_check: skipped").map_err(|e| e.to_string())?,
    }
    writeln!(out, "  galaxy_lock: {}", galaxy_doctor.lock_status).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  galaxy_lock_path: {}",
        galaxy_doctor.lock_path.display()
    )
    .map_err(|e| e.to_string())?;
    if let Some(error) = galaxy_doctor.lock_error.clone() {
        writeln!(out, "  galaxy_lock_error: {}", error).map_err(|e| e.to_string())?;
    }
    writeln!(
        out,
        "  galaxy_deps_root: {}",
        galaxy_doctor.deps_root.display()
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  galaxy_local_registry: {}",
        galaxy_doctor.local_registry_root.display()
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  galaxy_dependencies: {}",
        galaxy_doctor.dependencies.len()
    )
    .map_err(|e| e.to_string())?;
    for dependency in &galaxy_doctor.dependencies {
        writeln!(
            out,
            "  dep: {}={} local={} lock={} installed={}",
            dependency.name,
            dependency.version,
            crate::yes_no(dependency.local_available),
            crate::yes_no(dependency.locked),
            crate::yes_no(dependency.installed)
        )
        .map_err(|e| e.to_string())?;
    }
    writeln!(
        out,
        "  galaxy_imports: {}",
        project.manifest.galaxy_imports.len()
    )
    .map_err(|e| e.to_string())?;
    for item in &project.manifest.galaxy_imports {
        writeln!(
            out,
            "  galaxy_import: {}:{}",
            item.galaxy, item.library_module
        )
        .map_err(|e| e.to_string())?;
    }
    writeln!(
        out,
        "  galaxy_hidden_manual_only_library_modules: {}",
        hidden_manual_only_library_modules.len()
    )
    .map_err(|e| e.to_string())?;
    for item in &hidden_manual_only_library_modules {
        writeln!(out, "  galaxy_hidden_manual_only_library_module: {}", item)
            .map_err(|e| e.to_string())?;
    }
    match nova_profile.as_ref() {
        Some(profile) => {
            writeln!(out, "  ns_nova_profile: {}", profile.path.display())
                .map_err(|e| e.to_string())?;
            writeln!(out, "  ns_nova_framework: {}", profile.framework)
                .map_err(|e| e.to_string())?;
            writeln!(
                out,
                "  ns_nova_framework_schema: {}",
                profile.framework_schema
            )
            .map_err(|e| e.to_string())?;
        }
        None => writeln!(out, "  ns_nova_profile: <missing>").map_err(|e| e.to_string())?,
    }
    match nova_stdlib.as_ref() {
        Some(summary) => {
            writeln!(out, "  ns_nova_stdlib_manifest: {}", summary.path.display())
                .map_err(|e| e.to_string())?;
            writeln!(
                out,
                "  ns_nova_stdlib_sources: {}",
                summary.source_modules.len()
            )
            .map_err(|e| e.to_string())?;
            writeln!(
                out,
                "  ns_nova_stdlib_missing_sources: {}",
                summary.missing_modules.len()
            )
            .map_err(|e| e.to_string())?;
            for path in &summary.missing_modules {
                writeln!(out, "  ns_nova_stdlib_missing: {}", path.display())
                    .map_err(|e| e.to_string())?;
            }
        }
        None => writeln!(out, "  ns_nova_stdlib_manifest: <missing>").map_err(|e| e.to_string())?,
    }
    let mut next_steps = Vec::new();
    if !galaxy_manifest_exists {
        next_steps.push(
            "run `nuis galaxy init <project-dir>` if you want to package or share this project"
                .to_owned(),
        );
    }
    if let Some(profile) = nova_profile.as_ref() {
        if !galaxy_manifest_exists {
            next_steps.push(
                "run `nuis galaxy init <project-dir> --framework ns-nova` if this project should be packaged as an `ns-nova` framework project".to_owned(),
            );
        }
        if profile.family_schema.as_deref() == Some("ns-nova-family-v1")
            && profile.family_layers.is_empty()
        {
            next_steps.push(
                "fill `family_layers` in `ns-nova.toml` so the framework contract says whether this project is using `core`, `ui`, or `scene`".to_owned(),
            );
        }
    } else if nova_stdlib.is_some() {
        next_steps.push(
            "add `ns-nova.toml` if this project should carry explicit `ns-nova` framework metadata alongside the shared stdlib source asset catalog".to_owned(),
        );
    }
    if let Some(summary) = nova_stdlib.as_ref() {
        if summary.source_modules.is_empty() {
            next_steps.push(
                "fill `source_modules` in `stdlib/ns-nova/module.toml` so the framework declares its canonical `ns` source assets".to_owned(),
            );
        }
        if !summary.missing_modules.is_empty() {
            next_steps.push(
                "some `ns-nova` source modules declared in `stdlib/ns-nova/module.toml` are missing on disk; add them or remove stale entries from `source_modules`".to_owned(),
            );
        }
    }
    match lock_status.as_str() {
        "missing" if deps_len > 0 => next_steps.push(
            "run `nuis galaxy lock-deps <project-dir>` to create `nuis.galaxy.lock`".to_owned(),
        ),
        "invalid" => next_steps.push(
            "run `nuis galaxy verify-lock <project-dir>` after fixing the lock or regenerate it with `nuis galaxy lock-deps <project-dir>`".to_owned(),
        ),
        _ => {}
    }
    if any_lock_missing && deps_len > 0 && lock_status == "ok" {
        next_steps.push(
            "run `nuis galaxy lock-deps <project-dir>` to refresh the lock so it matches the manifest".to_owned(),
        );
    }
    if any_install_missing && lock_status == "ok" {
        next_steps.push(
            "run `nuis galaxy sync-deps <project-dir>` to materialize locked galaxy dependencies under `.nuis/deps/galaxy`".to_owned(),
        );
    }
    if any_local_missing && deps_len > 0 {
        next_steps.push(
            "some galaxy deps are not available locally; use `nuis galaxy list` to inspect the local registry or publish/install the missing packages first".to_owned(),
        );
    }
    if !hidden_manual_only_library_modules.is_empty() {
        next_steps.push(format!(
            "this project still has manual-only galaxy library modules that are not visible by default; run `nuis project-imports --apply-suggested <project-dir>` to write the recommended `galaxy_imports`, or edit `galaxy_imports = [...]` yourself if you want them in project scope: {}",
            hidden_manual_only_library_modules.join(", ")
        ));
    }
    if galaxy_check_invalid {
        next_steps.push(
            "run `nuis galaxy check <project-dir>` after fixing `galaxy.toml` or framework profile issues".to_owned(),
        );
    }
    if !plan.abi_resolution.explicit {
        next_steps.push(
            "run `nuis project-lock-abi <project-dir>` if you want to freeze the current ABI recommendations".to_owned(),
        );
    }
    if declared_tests.is_empty() {
        next_steps.push(
            "add `tests = [\"tests/smoke.ns\"]` to `nuis.toml` once you want `nuis test <project-dir>` to run explicit project test inputs".to_owned(),
        );
    }
    if !missing_tests.is_empty() {
        next_steps.push(
            "some declared project tests are missing on disk; add those `.ns` files or remove stale entries from `tests = [...]` in `nuis.toml`".to_owned(),
        );
    }
    if next_steps.is_empty() {
        writeln!(out, "  next_steps: none").map_err(|e| e.to_string())?;
    } else {
        writeln!(out, "  next_steps: {}", next_steps.len()).map_err(|e| e.to_string())?;
        for step in next_steps {
            writeln!(out, "  next: {}", step).map_err(|e| e.to_string())?;
        }
    }
    if let Some(error) = lock_error {
        writeln!(
            out,
            "  note: lock verification failed before suggestions were computed: {}",
            error
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
