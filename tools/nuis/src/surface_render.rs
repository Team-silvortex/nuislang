use std::path::Path;

fn load_link_plan(output_dir: &Path) -> Option<nuisc::linker::LinkPlan> {
    let manifest = output_dir.join("nuis.build.manifest.toml");
    if !manifest.exists() {
        return None;
    }
    nuisc::linker::build_link_plan_from_manifest(&manifest).ok()
}

fn append_link_plan_text_fields(
    lines: &mut Vec<String>,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    lines.push(format!(
        "  link_plan_available: {}",
        crate::yes_no(link_plan.is_some())
    ));
    if let Some(plan) = link_plan {
        lines.push(format!("  link_plan_final_stage: {}", plan.final_stage.kind));
        lines.push(format!(
            "  link_plan_final_driver: {}",
            plan.final_stage.driver
        ));
        lines.push(format!(
            "  link_plan_final_link_mode: {}",
            plan.final_stage.link_mode
        ));
        lines.push(format!(
            "  link_plan_final_output: {}",
            plan.final_stage.output_path
        ));
        lines.push(format!(
            "  link_plan_domain_units: {}",
            plan.domain_units.len()
        ));
    } else {
        lines.push("  link_plan_final_stage: <unavailable>".to_owned());
        lines.push("  link_plan_final_driver: <unavailable>".to_owned());
        lines.push("  link_plan_final_link_mode: <unavailable>".to_owned());
        lines.push("  link_plan_final_output: <unavailable>".to_owned());
        lines.push("  link_plan_domain_units: 0".to_owned());
    }
}

fn link_plan_json_fields(link_plan: Option<&nuisc::linker::LinkPlan>) -> Vec<String> {
    vec![
        crate::json_bool_field("link_plan_available", link_plan.is_some()),
        crate::json_optional_string_field(
            "link_plan_final_stage",
            link_plan.map(|plan| plan.final_stage.kind.as_str()),
        ),
        crate::json_optional_string_field(
            "link_plan_final_driver",
            link_plan.map(|plan| plan.final_stage.driver.as_str()),
        ),
        crate::json_optional_string_field(
            "link_plan_final_link_mode",
            link_plan.map(|plan| plan.final_stage.link_mode.as_str()),
        ),
        crate::json_optional_string_field(
            "link_plan_final_output",
            link_plan.map(|plan| plan.final_stage.output_path.as_str()),
        ),
        crate::json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
    ]
}

pub(crate) fn render_project_status_text_summary(input: &Path) -> Result<Vec<String>, String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let public_surface = crate::public_surface_records(&project);
    let galaxy_lock_status = crate::galaxy::verify_project_lock(input);
    let galaxy_manifest_path = project.root.join("galaxy.toml");
    let include_galaxy_flow =
        galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
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
    let galaxy_check = if galaxy_manifest_path.exists() {
        Some(crate::galaxy::check(&project.root))
    } else {
        None
    };
    let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
    let galaxy_doctor = crate::galaxy::doctor_project(&project.root)?;
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
    let mut lines = vec![
        format!("project status: {}", project.manifest.name),
        format!("  root: {}", project.root.display()),
        format!("  manifest: {}", project.manifest_path.display()),
        format!("  entry: {}", project.manifest.entry),
        format!("  frontdoor.source_kind: {}", frontdoor.source_kind),
        format!("  frontdoor.workflow_kind: {}", frontdoor.workflow_kind),
        format!("  frontdoor.workflow_brief: {}", frontdoor.workflow_brief),
        format!("  frontdoor.workflow_samples: {}", frontdoor.workflow_samples),
        format!(
            "  frontdoor.recommended_next_step: {}",
            frontdoor.recommended_next_step
        ),
        format!(
            "  frontdoor.recommended_command: {}",
            frontdoor.recommended_command
        ),
        format!(
            "  frontdoor.recommended_reason: {}",
            frontdoor.recommended_reason
        ),
        format!("  recommended_next_step: {}", frontdoor.recommended_next_step),
        format!("  recommended_command: {}", frontdoor.recommended_command),
        format!("  recommended_reason: {}", frontdoor.recommended_reason),
        format!("  artifact_output_dir: {}", artifact_output_dir.display()),
        format!("  artifact_ready_to_run: {}", crate::yes_no(artifact_report.ready_to_run)),
        format!(
            "  artifact_recommended_next_step: {}",
            artifact_report.recommended_next_step
        ),
        format!(
            "  artifact_recommended_command: {}",
            artifact_report.recommended_command
        ),
        format!("  modules: {}", project.modules.len()),
        format!(
            "  public_surface: {}",
            crate::describe_public_surface(&public_surface)
        ),
        format!(
            "  public_surface_modules: {}",
            crate::describe_public_surface_modules(&public_surface)
        ),
        format!("  links: {}", project.manifest.links.len()),
        format!(
            "  project_plan: {}",
            nuisc::project::describe_project_compilation_plan(&plan)
        ),
        format!(
            "  project_plan_dependencies: {}",
            if plan.dependencies.is_empty() {
                "<none>".to_owned()
            } else {
                plan.dependencies
                    .iter()
                    .map(|item| format!("{}:{}={} ({})", item.category, item.name, item.version, item.source))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ),
        format!(
            "  project_plan_dependency_categories: {}",
            nuisc::project::describe_project_dependency_categories(&plan)
        ),
        format!(
            "  project_plan_synthetic_input: {} ({})",
            plan.synthetic_input.path.display(),
            plan.synthetic_input.kind
        ),
        format!("  project_plan_outputs: {}", plan.output_intents.len()),
        format!(
            "  project_plan_output_categories: {}",
            nuisc::project::describe_project_output_intent_categories(&plan)
        ),
        format!("  project_organization_entry: {}", plan.organization.entry),
        format!("  project_exchange_routes: {}", plan.exchanges.routes.len()),
        format!(
            "  project_exchange_route_classes: {}",
            nuisc::project::describe_project_exchange_route_classes(&plan)
        ),
        format!("  tests: {}", declared_tests.len()),
    ];
    append_link_plan_text_fields(&mut lines, link_plan.as_ref());
    for path in &declared_tests {
        lines.push(format!("  test: {} exists={}", path.display(), crate::yes_no(path.exists())));
    }
    lines.push(format!("  project_compile_workflow: {}", nuisc::project_compile_workflow_brief()));
    lines.push(format!("  project_compile_samples: {}", nuisc::project_compile_samples_brief()));
    lines.push(format!("  project_test_workflow: {}", nuisc::project_test_workflow_brief()));
    if include_galaxy_flow {
        lines.push(format!(
            "  project_galaxy_workflow: {}",
            nuisc::project_galaxy_workflow_brief()
        ));
    }
    lines.push(format!("  domains: {}", plan.organization.domains.join(", ")));
    lines.push(format!(
        "  abi_mode: {}",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    ));
    for item in &project.manifest.galaxy_dependencies {
        lines.push(format!("  galaxy: {}={}", item.name, item.version));
    }
    lines.push(format!(
        "  galaxy_imports: {}",
        project.manifest.galaxy_imports.len()
    ));
    for item in &project.manifest.galaxy_imports {
        lines.push(format!(
            "  galaxy_import: {}:{}",
            item.galaxy, item.library_module
        ));
    }
    lines.push(format!(
        "  galaxy_hidden_manual_only_library_modules: {}",
        hidden_manual_only_library_modules.len()
    ));
    for item in &hidden_manual_only_library_modules {
        lines.push(format!("  galaxy_hidden_manual_only_library_module: {}", item));
    }
    let lock_path = project.root.join("nuis.galaxy.lock");
    match galaxy_lock_status {
        Ok(lock) => {
            lines.push("  galaxy_lock: ok".to_owned());
            lines.push(format!("  galaxy_lock_path: {}", lock.path.display()));
            lines.push(format!("  galaxy_lock_dependencies: {}", lock.entries.len()));
            let declared = project
                .manifest
                .galaxy_dependencies
                .iter()
                .map(|item| format!("{}={}", item.name, item.version))
                .collect::<std::collections::BTreeSet<_>>();
            let locked = lock
                .entries
                .iter()
                .map(|item| format!("{}={}", item.name, item.version))
                .collect::<std::collections::BTreeSet<_>>();
            lines.push(format!(
                "  galaxy_lock_matches_manifest: {}",
                if declared == locked { "yes" } else { "no" }
            ));
            for item in lock.entries {
                lines.push(format!(
                    "  galaxy_lock_entry: {}={} {}",
                    item.name, item.version, item.bundle_fnv1a64
                ));
            }
        }
        Err(error) if lock_path.exists() => {
            lines.push("  galaxy_lock: invalid".to_owned());
            lines.push(format!("  galaxy_lock_path: {}", lock_path.display()));
            lines.push(format!("  galaxy_lock_error: {}", error));
        }
        Err(_) => {
            lines.push("  galaxy_lock: missing".to_owned());
            lines.push(format!("  galaxy_lock_path: {}", lock_path.display()));
        }
    }
    Ok(lines)
}

pub(crate) fn render_project_doctor_text_summary(input: &Path) -> Result<Vec<String>, String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
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
    let mut lines = vec![
        format!("project doctor: {}", project.manifest.name),
        format!("  root: {}", project.root.display()),
        format!("  manifest: {}", project.manifest_path.display()),
        format!("  entry: {}", project.manifest.entry),
        format!("  frontdoor.source_kind: {}", frontdoor.source_kind),
        format!("  frontdoor.workflow_kind: {}", frontdoor.workflow_kind),
        format!("  frontdoor.workflow_brief: {}", frontdoor.workflow_brief),
        format!("  frontdoor.workflow_samples: {}", frontdoor.workflow_samples),
        format!(
            "  frontdoor.recommended_next_step: {}",
            frontdoor.recommended_next_step
        ),
        format!(
            "  frontdoor.recommended_command: {}",
            frontdoor.recommended_command
        ),
        format!(
            "  frontdoor.recommended_reason: {}",
            frontdoor.recommended_reason
        ),
        format!("  recommended_next_step: {}", frontdoor.recommended_next_step),
        format!("  recommended_command: {}", frontdoor.recommended_command),
        format!("  recommended_reason: {}", frontdoor.recommended_reason),
        format!("  artifact_output_dir: {}", artifact_output_dir.display()),
        format!("  artifact_ready_to_run: {}", crate::yes_no(artifact_report.ready_to_run)),
        format!(
            "  artifact_recommended_next_step: {}",
            artifact_report.recommended_next_step
        ),
        format!(
            "  artifact_recommended_command: {}",
            artifact_report.recommended_command
        ),
        format!("  modules: {}", project.modules.len()),
        format!(
            "  public_surface: {}",
            crate::describe_public_surface(&public_surface)
        ),
        format!(
            "  public_surface_modules: {}",
            crate::describe_public_surface_modules(&public_surface)
        ),
        format!("  links: {}", project.manifest.links.len()),
        format!(
            "  project_plan: {}",
            nuisc::project::describe_project_compilation_plan(&plan)
        ),
        format!("  tests_declared: {}", declared_tests.len()),
        format!("  tests_missing: {}", missing_tests.len()),
    ];
    append_link_plan_text_fields(&mut lines, link_plan.as_ref());
    for path in &declared_tests {
        lines.push(format!("  test: {} exists={}", path.display(), crate::yes_no(path.exists())));
    }
    lines.push(format!("  project_compile_workflow: {}", nuisc::project_compile_workflow_brief()));
    lines.push(format!("  project_compile_samples: {}", nuisc::project_compile_samples_brief()));
    lines.push(format!("  project_test_workflow: {}", nuisc::project_test_workflow_brief()));
    if include_galaxy_flow {
        lines.push(format!(
            "  project_galaxy_workflow: {}",
            nuisc::project_galaxy_workflow_brief()
        ));
    }
    lines.push(format!(
        "  abi_mode: {}",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    ));
    lines.push(format!("  abi_checks: {}", abi_checks.len()));
    lines.push(format!("  registry_checks: {}", registry_checks.len()));
    lines.push(format!("  lowering_checks: {}", lowering_checks.len()));
    lines.push(format!(
        "  galaxy_manifest: {}",
        if galaxy_manifest_exists {
            galaxy_manifest_path.display().to_string()
        } else {
            "<missing>".to_owned()
        }
    ));
    match galaxy_check {
        Some(Ok(checked)) => {
            lines.push("  galaxy_check: ok".to_owned());
            lines.push(format!("  galaxy_package_kind: {}", checked.manifest.package_kind));
            lines.push(format!(
                "  galaxy_framework: {}",
                checked.manifest.framework.as_deref().unwrap_or("<none>")
            ));
            lines.push(format!("  galaxy_include_files: {}", checked.include_files.len()));
        }
        Some(Err(error)) => {
            lines.push("  galaxy_check: invalid".to_owned());
            lines.push(format!("  galaxy_error: {}", error));
        }
        None => lines.push("  galaxy_check: skipped".to_owned()),
    }
    lines.push(format!("  galaxy_lock: {}", galaxy_doctor.lock_status));
    lines.push(format!("  galaxy_lock_path: {}", galaxy_doctor.lock_path.display()));
    if let Some(error) = galaxy_doctor.lock_error.clone() {
        lines.push(format!("  galaxy_lock_error: {}", error));
    }
    lines.push(format!("  galaxy_deps_root: {}", galaxy_doctor.deps_root.display()));
    lines.push(format!(
        "  galaxy_local_registry: {}",
        galaxy_doctor.local_registry_root.display()
    ));
    lines.push(format!("  galaxy_dependencies: {}", galaxy_doctor.dependencies.len()));
    for dependency in &galaxy_doctor.dependencies {
        lines.push(format!(
            "  dep: {}={} local={} lock={} installed={}",
            dependency.name,
            dependency.version,
            crate::yes_no(dependency.local_available),
            crate::yes_no(dependency.locked),
            crate::yes_no(dependency.installed)
        ));
    }
    lines.push(format!(
        "  galaxy_imports: {}",
        project.manifest.galaxy_imports.len()
    ));
    for item in &project.manifest.galaxy_imports {
        lines.push(format!(
            "  galaxy_import: {}:{}",
            item.galaxy, item.library_module
        ));
    }
    lines.push(format!(
        "  galaxy_hidden_manual_only_library_modules: {}",
        hidden_manual_only_library_modules.len()
    ));
    for item in &hidden_manual_only_library_modules {
        lines.push(format!("  galaxy_hidden_manual_only_library_module: {}", item));
    }
    match nova_profile.as_ref() {
        Some(profile) => {
            lines.push(format!("  ns_nova_profile: {}", profile.path.display()));
            lines.push(format!("  ns_nova_framework: {}", profile.framework));
            lines.push(format!("  ns_nova_framework_schema: {}", profile.framework_schema));
        }
        None => lines.push("  ns_nova_profile: <missing>".to_owned()),
    }
    match nova_stdlib.as_ref() {
        Some(summary) => {
            lines.push(format!("  ns_nova_stdlib_manifest: {}", summary.path.display()));
            lines.push(format!("  ns_nova_stdlib_sources: {}", summary.source_modules.len()));
            lines.push(format!(
                "  ns_nova_stdlib_missing_sources: {}",
                summary.missing_modules.len()
            ));
            for path in &summary.missing_modules {
                lines.push(format!("  ns_nova_stdlib_missing: {}", path.display()));
            }
        }
        None => lines.push("  ns_nova_stdlib_manifest: <missing>".to_owned()),
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
        lines.push("  next_steps: none".to_owned());
    } else {
        lines.push(format!("  next_steps: {}", next_steps.len()));
        for step in next_steps {
            lines.push(format!("  next: {}", step));
        }
    }
    if let Some(error) = lock_error {
        lines.push(format!(
            "  note: lock verification failed before suggestions were computed: {}",
            error
        ));
    }
    Ok(lines)
}
pub(crate) fn render_scheduler_view_json(input: &Path) -> Result<String, String> {
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
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
        let galaxy_check = if galaxy_manifest_path.exists() {
            Some(crate::galaxy::check(&project.root))
        } else {
            None
        };
        let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
        let galaxy_doctor = crate::galaxy::doctor_project(&project.root)?;
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
        let mut domains = Vec::new();
        for item in &plan.abi_resolution.requirements {
            domains.push(crate::scheduler_view_domain_record(
                &item.domain,
                None,
                Some(item.abi.clone()),
            )?);
        }
        let domain_json = domains
            .iter()
            .map(crate::scheduler_view_domain_record_json)
            .collect::<Vec<_>>()
            .join(",");
        let mut fields = vec![
            crate::json_field("source_kind", "project"),
            crate::json_field("input", &input.display().to_string()),
            crate::json_field("project", &project.manifest.name),
            crate::json_field(
                "abi_mode",
                if plan.abi_resolution.explicit {
                    "explicit"
                } else {
                    "auto-recommended"
                },
            ),
        ];
        fields.extend(crate::json_surface::workflow_contract_json_fields(
            &frontdoor,
            false,
            false,
            false,
            false,
        ));
        fields.extend(crate::json_surface::project_plan_json_fields(&plan));
        return Ok(format!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json));
    }

    let artifacts = nuisc::pipeline::compile_source_path(&input)?;
    let manifests = nuisc::registry::load_required_manifests(
        std::path::Path::new("nustar-packages"),
        &artifacts.yir,
    )?;
    let frontdoor = crate::single_source_frontdoor_surface();
    let mut domains = Vec::new();
    for manifest in manifests {
        domains.push(crate::scheduler_view_domain_record(
            &manifest.domain_family,
            Some(manifest.package_id),
            None,
        )?);
    }
    let domain_json = domains
        .iter()
        .map(crate::scheduler_view_domain_record_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        crate::json_field("source_kind", "single-file"),
        crate::json_field("input", &input.display().to_string()),
        crate::json_field("ast_domain", &artifacts.ast.domain),
        crate::json_field("ast_unit", &artifacts.ast.unit),
        crate::json_object_field(
            "frontdoor",
            &crate::workflow_frontdoor_json_fields(&frontdoor),
        ),
        crate::json_field("workflow_kind", frontdoor.workflow_kind),
        crate::json_field("workflow_brief", frontdoor.workflow_brief),
        crate::json_field("workflow_samples", frontdoor.workflow_samples),
        crate::json_field("recommended_next_step", frontdoor.recommended_next_step),
        crate::json_field("recommended_command", frontdoor.recommended_command),
        crate::json_field("recommended_reason", frontdoor.recommended_reason),
    ];
    Ok(format!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json))
}

pub(crate) fn render_project_status_json(input: &Path) -> Result<String, String> {
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let public_surface = crate::public_surface_records(&project);
    let galaxy_lock_status = crate::galaxy::verify_project_lock(&input);
    let galaxy_manifest_path = project.root.join("galaxy.toml");
    let include_galaxy_flow =
        galaxy_manifest_path.exists() || !project.manifest.galaxy_dependencies.is_empty();
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
    let galaxy_check = if galaxy_manifest_path.exists() {
        Some(crate::galaxy::check(&project.root))
    } else {
        None
    };
    let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
    let galaxy_doctor = crate::galaxy::doctor_project(&project.root)?;
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
    let test_json = declared_tests
        .iter()
        .map(|path| {
            format!(
                "{{{},{}}}",
                crate::json_field("path", &path.display().to_string()),
                crate::json_bool_field("exists", path.exists())
            )
        })
        .collect::<Vec<_>>();
    let domain_json = crate::project_plan_domains_json(&plan)?;
    let public_surface_json = crate::public_surface_json(&public_surface);
    let mut fields = vec![
        crate::json_field("source_kind", "project"),
        crate::json_field("input", &input.display().to_string()),
        crate::json_field("project", &project.manifest.name),
        crate::json_field("root", &project.root.display().to_string()),
        crate::json_field("manifest", &project.manifest_path.display().to_string()),
        crate::json_field("entry", &project.manifest.entry),
        crate::json_usize_field("modules", project.modules.len()),
        crate::json_usize_field("links", project.manifest.links.len()),
    ];
    fields.extend(crate::json_surface::public_surface_summary_json_fields(
        &public_surface,
    ));
    fields.extend(crate::json_surface::project_plan_json_fields(&plan));
    fields.push(crate::json_usize_field("tests_declared", declared_tests.len()));
    fields.push(crate::json_field(
        "artifact_output_dir",
        &artifact_output_dir.display().to_string(),
    ));
    fields.push(crate::json_bool_field(
        "artifact_ready_to_run",
        artifact_report.ready_to_run,
    ));
    fields.push(crate::json_field(
        "artifact_recommended_next_step",
        &artifact_report.recommended_next_step,
    ));
    fields.push(crate::json_field(
        "artifact_recommended_command",
        &artifact_report.recommended_command,
    ));
    fields.extend(link_plan_json_fields(link_plan.as_ref()));
    fields.extend(crate::project_workflow_json_fields(
        &frontdoor,
        include_galaxy_flow,
    ));
    fields.push(crate::json_field(
        "abi_mode",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        },
    ));
    fields.push(crate::json_string_array_field(
        "galaxy_dependencies",
        &project
            .manifest
            .galaxy_dependencies
            .iter()
            .map(|item| format!("{}={}", item.name, item.version))
            .collect::<Vec<_>>(),
    ));
    fields.push(crate::json_usize_field(
        "galaxy_imports_count",
        project.manifest.galaxy_imports.len(),
    ));
    fields.push(crate::json_string_array_field(
        "galaxy_imports",
        &project
            .manifest
            .galaxy_imports
            .iter()
            .map(|item| format!("{}:{}", item.galaxy, item.library_module))
            .collect::<Vec<_>>(),
    ));
    fields.push(crate::json_usize_field(
        "galaxy_hidden_manual_only_library_modules_count",
        hidden_manual_only_library_modules.len(),
    ));
    fields.push(crate::json_string_array_field(
        "galaxy_hidden_manual_only_library_modules",
        &hidden_manual_only_library_modules,
    ));
    let lock_path = project.root.join("nuis.galaxy.lock");
    let declared_galaxy_dependencies = project
        .manifest
        .galaxy_dependencies
        .iter()
        .map(|item| format!("{}={}", item.name, item.version))
        .collect::<Vec<_>>();
    fields.extend(crate::json_surface::galaxy_lock_json_fields(
        galaxy_lock_status,
        &lock_path,
        &declared_galaxy_dependencies,
    ));
    fields.push(crate::json_object_array_field("tests", &test_json));
    fields.push(crate::json_object_array_field(
        "public_surface_records",
        &public_surface_json,
    ));
    Ok(format!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json))
}

pub(crate) fn render_project_doctor_json(input: &Path) -> Result<String, String> {
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
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
        if profile.render_schema.as_deref() == Some("ns-nova-render-v1")
            && (profile.render_owner_unit.is_none()
                || profile.render_bridge_unit.is_none()
                || profile.render_surface_unit.is_none())
        {
            next_steps.push(
                "fill `render_owner_unit`, `render_bridge_unit`, and `render_surface_unit` in `ns-nova.toml` to complete the render contract".to_owned(),
            );
        }
        if profile.selection_schema.as_deref() == Some("ns-nova-selection-v1")
            && (profile.selection_owner_unit.is_none()
                || profile.selection_bridge_unit.is_none()
                || profile.selection_render_unit.is_none()
                || profile.selection_controls.is_empty())
        {
            next_steps.push(
                "fill the `selection_*` units and `selection_controls` in `ns-nova.toml` to complete the shared selection contract".to_owned(),
            );
        }
        if profile.stdlib_schema.as_deref() == Some("ns-nova-stdlib-v1")
            && (profile.stdlib_manifest.is_none() || profile.stdlib_sources.is_empty())
        {
            next_steps.push(
                "fill `stdlib_manifest` and `stdlib_sources` in `ns-nova.toml` so the framework profile points at its canonical stdlib source assets".to_owned(),
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
        if let Some(profile) = nova_profile.as_ref() {
            if profile.stdlib_sources.len() != summary.source_modules.len() {
                next_steps.push(
                    "refresh `ns-nova.toml` so its `stdlib_sources` count matches `stdlib/ns-nova/module.toml`".to_owned(),
                );
            }
        }
    }
    match lock_status.as_str() {
        "missing" if deps_len > 0 => {
            next_steps.push(
                "run `nuis galaxy lock-deps <project-dir>` to create `nuis.galaxy.lock`".to_owned(),
            );
        }
        "invalid" => {
            next_steps.push(
                "run `nuis galaxy verify-lock <project-dir>` after fixing the lock or regenerate it with `nuis galaxy lock-deps <project-dir>`".to_owned(),
            );
        }
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
    let domain_json = crate::project_plan_domains_json(&plan)?;
    let public_surface_json = crate::public_surface_json(&public_surface);
    let tests_json = declared_tests
        .iter()
        .map(|path| {
            format!(
                "{{{},{}}}",
                crate::json_field("path", &path.display().to_string()),
                crate::json_bool_field("exists", path.exists())
            )
        })
        .collect::<Vec<_>>();
    let dependency_json = galaxy_doctor
        .dependencies
        .iter()
        .map(|dependency| {
            format!(
                "{{{},{},{},{},{}}}",
                crate::json_field("name", &dependency.name),
                crate::json_field("version", &dependency.version),
                crate::json_bool_field("local_available", dependency.local_available),
                crate::json_bool_field("locked", dependency.locked),
                crate::json_bool_field("installed", dependency.installed),
            )
        })
        .collect::<Vec<_>>();
    let galaxy_manifest_display = if galaxy_manifest_exists {
        galaxy_manifest_path.display().to_string()
    } else {
        "<missing>".to_owned()
    };
    let mut fields = vec![
        crate::json_field("source_kind", "project"),
        crate::json_field("input", &input.display().to_string()),
        crate::json_field("project", &project.manifest.name),
        crate::json_field("root", &project.root.display().to_string()),
        crate::json_field("manifest", &project.manifest_path.display().to_string()),
        crate::json_field("entry", &project.manifest.entry),
        crate::json_usize_field("modules", project.modules.len()),
        crate::json_usize_field("links", project.manifest.links.len()),
    ];
    fields.extend(crate::json_surface::public_surface_summary_json_fields(
        &public_surface,
    ));
    fields.extend(crate::json_surface::project_plan_json_fields(&plan));
    fields.push(crate::json_usize_field("tests_declared", declared_tests.len()));
    fields.push(crate::json_usize_field("tests_missing", missing_tests.len()));
    fields.push(crate::json_field(
        "artifact_output_dir",
        &artifact_output_dir.display().to_string(),
    ));
    fields.push(crate::json_bool_field(
        "artifact_ready_to_run",
        artifact_report.ready_to_run,
    ));
    fields.push(crate::json_field(
        "artifact_recommended_next_step",
        &artifact_report.recommended_next_step,
    ));
    fields.push(crate::json_field(
        "artifact_recommended_command",
        &artifact_report.recommended_command,
    ));
    fields.extend(link_plan_json_fields(link_plan.as_ref()));
    fields.extend(crate::project_workflow_json_fields(
        &frontdoor,
        include_galaxy_flow,
    ));
    fields.push(crate::json_field(
        "abi_mode",
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        },
    ));
    fields.extend(crate::json_surface::project_check_summary_json_fields(
        &abi_checks,
        &registry_checks,
        &lowering_checks,
    ));
    fields.push(crate::json_field("galaxy_manifest", &galaxy_manifest_display));
    match galaxy_check {
        Some(Ok(checked)) => {
            fields.push(crate::json_field("galaxy_check_status", "ok"));
            fields.push(crate::json_field(
                "galaxy_package_kind",
                &checked.manifest.package_kind,
            ));
            fields.push(crate::json_field(
                "galaxy_framework",
                checked.manifest.framework.as_deref().unwrap_or("<none>"),
            ));
            fields.push(crate::json_usize_field(
                "galaxy_include_files",
                checked.include_files.len(),
            ));
        }
        Some(Err(error)) => {
            fields.push(crate::json_field("galaxy_check_status", "invalid"));
            fields.push(crate::json_field("galaxy_error", &error));
        }
        None => {
            fields.push(crate::json_field("galaxy_check_status", "skipped"));
        }
    }
    fields.push(crate::json_field("galaxy_lock_status", &galaxy_doctor.lock_status));
    fields.push(crate::json_field(
        "galaxy_lock_path",
        &galaxy_doctor.lock_path.display().to_string(),
    ));
    if let Some(error) = galaxy_doctor.lock_error.as_deref() {
        fields.push(crate::json_field("galaxy_lock_error", error));
    }
    fields.push(crate::json_field(
        "galaxy_deps_root",
        &galaxy_doctor.deps_root.display().to_string(),
    ));
    fields.push(crate::json_field(
        "galaxy_local_registry",
        &galaxy_doctor.local_registry_root.display().to_string(),
    ));
    fields.push(crate::json_usize_field(
        "galaxy_dependencies_count",
        galaxy_doctor.dependencies.len(),
    ));
    fields.push(crate::json_usize_field(
        "galaxy_imports_count",
        project.manifest.galaxy_imports.len(),
    ));
    fields.push(crate::json_string_array_field(
        "galaxy_imports",
        &project
            .manifest
            .galaxy_imports
            .iter()
            .map(|item| format!("{}:{}", item.galaxy, item.library_module))
            .collect::<Vec<_>>(),
    ));
    fields.push(crate::json_usize_field(
        "galaxy_hidden_manual_only_library_modules_count",
        hidden_manual_only_library_modules.len(),
    ));
    fields.push(crate::json_string_array_field(
        "galaxy_hidden_manual_only_library_modules",
        &hidden_manual_only_library_modules,
    ));
    fields.push(crate::json_optional_string_field(
        "ns_nova_profile",
        nova_profile
            .as_ref()
            .map(|profile| profile.path.display().to_string())
            .as_deref(),
    ));
    fields.push(crate::json_optional_string_field(
        "ns_nova_stdlib_manifest",
        nova_stdlib
            .as_ref()
            .map(|summary| summary.path.display().to_string())
            .as_deref(),
    ));
    if let Some(error) = lock_error.as_deref() {
        fields.push(crate::json_field("note", error));
    }
    fields.push(crate::json_string_array_field("next_steps", &next_steps));
    fields.push(crate::json_object_array_field("tests", &tests_json));
    fields.push(crate::json_object_array_field(
        "public_surface_records",
        &public_surface_json,
    ));
    fields.push(crate::json_object_array_field(
        "abi_checks",
        &crate::project_abi_checks_json(&abi_checks),
    ));
    fields.push(crate::json_object_array_field(
        "registry_checks",
        &crate::project_domain_registry_checks_json(&registry_checks),
    ));
    fields.push(crate::json_object_array_field(
        "lowering_checks",
        &crate::project_lowering_checks_json(&lowering_checks),
    ));
    fields.push(crate::json_object_array_field(
        "galaxy_dependencies",
        &dependency_json,
    ));
    Ok(format!("{{{},\"domains\":[{}]}}", fields.join(","), domain_json))
}
