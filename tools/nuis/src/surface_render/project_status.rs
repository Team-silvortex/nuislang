use super::*;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_project_status_text_summary(input: &Path) -> Result<Vec<String>, String> {
    let mut out = String::new();
    write_project_status_text_summary(&mut out, input)?;
    Ok(out.lines().map(str::to_owned).collect())
}

pub(crate) fn write_project_status_text_summary<W: fmt::Write>(
    out: &mut W,
    input: &Path,
) -> Result<(), String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let text_handle_rewrite = nuisc::project::summarize_project_text_handle_rewrites(&project)?;
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
    let closure_summary = project_closure_summary(
        "project-status",
        "project-status-link-plan",
        artifact_report.ready_to_run,
        missing_tests.len(),
        &frontdoor,
        link_plan.as_ref(),
    );
    writeln!(out, "project status: {}", project.manifest.name).map_err(|e| e.to_string())?;
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
    writeln!(out, "  closure_summary_status: {}", closure_summary.status)
        .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  closure_summary_primary_blocker: {}",
        closure_summary
            .primary_blocker
            .as_deref()
            .unwrap_or("<none>")
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  closure_summary_next_action: {}",
        closure_summary.next_action
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
        "  artifact_payload_decoder_manifest_available: {}",
        crate::yes_no(artifact_report.payload_decoder_manifest.available)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_payload_decoder_manifest_status: {}",
        artifact_report.payload_decoder_manifest.status
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_payload_decoder_manifest_record_count: {}",
        artifact_report.payload_decoder_manifest.record_count
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  artifact_payload_decoder_manifest_invalid_record_count: {}",
        artifact_report
            .payload_decoder_manifest
            .invalid_record_count
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
    write!(out, "  project_plan_dependencies: ").map_err(|e| e.to_string())?;
    if plan.dependencies.is_empty() {
        writeln!(out, "<none>").map_err(|e| e.to_string())?;
    } else {
        for (index, item) in plan.dependencies.iter().enumerate() {
            if index > 0 {
                write!(out, ", ").map_err(|e| e.to_string())?;
            }
            write!(
                out,
                "{}:{}={} ({})",
                item.category, item.name, item.version, item.source
            )
            .map_err(|e| e.to_string())?;
        }
        writeln!(out).map_err(|e| e.to_string())?;
    }
    writeln!(
        out,
        "  project_plan_dependency_categories: {}",
        nuisc::project::describe_project_dependency_categories(&plan)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_plan_synthetic_input: {} ({})",
        plan.synthetic_input.path.display(),
        plan.synthetic_input.kind
    )
    .map_err(|e| e.to_string())?;
    writeln!(out, "  project_plan_outputs: {}", plan.output_intents.len())
        .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_plan_output_categories: {}",
        nuisc::project::describe_project_output_intent_categories(&plan)
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_organization_entry: {}",
        plan.organization.entry
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_exchange_routes: {}",
        plan.exchanges.routes.len()
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        out,
        "  project_exchange_route_classes: {}",
        nuisc::project::describe_project_exchange_route_classes(&plan)
    )
    .map_err(|e| e.to_string())?;
    writeln!(out, "  tests: {}", declared_tests.len()).map_err(|e| e.to_string())?;
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
    writeln!(out, "  domains: {}", plan.organization.domains.join(", "))
        .map_err(|e| e.to_string())?;
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
    for item in &project.manifest.galaxy_dependencies {
        writeln!(out, "  galaxy: {}={}", item.name, item.version).map_err(|e| e.to_string())?;
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
    let lock_path = project.root.join("nuis.galaxy.lock");
    match galaxy_lock_status {
        Ok(lock) => {
            writeln!(out, "  galaxy_lock: ok").map_err(|e| e.to_string())?;
            writeln!(out, "  galaxy_lock_path: {}", lock.path.display())
                .map_err(|e| e.to_string())?;
            writeln!(out, "  galaxy_lock_dependencies: {}", lock.entries.len())
                .map_err(|e| e.to_string())?;
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
            writeln!(
                out,
                "  galaxy_lock_matches_manifest: {}",
                if declared == locked { "yes" } else { "no" }
            )
            .map_err(|e| e.to_string())?;
            for item in lock.entries {
                writeln!(
                    out,
                    "  galaxy_lock_entry: {}={} {}",
                    item.name, item.version, item.bundle_fnv1a64
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Err(error) if lock_path.exists() => {
            writeln!(out, "  galaxy_lock: invalid").map_err(|e| e.to_string())?;
            writeln!(out, "  galaxy_lock_path: {}", lock_path.display())
                .map_err(|e| e.to_string())?;
            writeln!(out, "  galaxy_lock_error: {}", error).map_err(|e| e.to_string())?;
        }
        Err(_) => {
            writeln!(out, "  galaxy_lock: missing").map_err(|e| e.to_string())?;
            writeln!(out, "  galaxy_lock_path: {}", lock_path.display())
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

pub(crate) fn render_project_status_json(input: &Path) -> Result<String, String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let text_handle_rewrite = nuisc::project::summarize_project_text_handle_rewrites(&project)?;
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
    let closure_summary = project_closure_summary(
        "project-status",
        "project-status-link-plan",
        artifact_report.ready_to_run,
        missing_tests.len(),
        &frontdoor,
        link_plan.as_ref(),
    );
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
    let galaxy_surface_ids = project
        .resolved_galaxies
        .iter()
        .flat_map(|dependency| dependency.surfaces.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let galaxy_records = project
        .resolved_galaxies
        .iter()
        .map(|dependency| {
            format!(
                "{{{},{},{},{}}}",
                crate::json_field("name", &dependency.name),
                crate::json_field("package_id", &dependency.package_id),
                crate::json_string_array_field("surfaces", &dependency.surfaces),
                crate::json_string_array_field("library_modules", &dependency.library_modules)
            )
        })
        .collect::<Vec<_>>();
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_field("source_kind", "project"),
            crate::json_field("input", &input.display().to_string()),
            crate::json_field("project", &project.manifest.name),
            crate::json_field("root", &project.root.display().to_string()),
            crate::json_field("manifest", &project.manifest_path.display().to_string()),
            crate::json_field("entry", &project.manifest.entry),
            crate::json_usize_field("modules", project.modules.len()),
            crate::json_usize_field("links", project.manifest.links.len()),
            crate::json_usize_field(
                "text_handle_rewrite_helper_hits",
                text_handle_rewrite.helper_hits,
            ),
            crate::json_usize_field(
                "text_handle_rewrite_local_hits",
                text_handle_rewrite.local_hits,
            ),
            crate::json_usize_field(
                "text_handle_rewrite_total_hits",
                text_handle_rewrite.total_hits(),
            ),
        ],
    );
    append_json_field_strings(&mut out, closure_summary.json_fields());
    crate::json_surface::append_public_surface_summary_json_fields(&mut out, &public_surface);
    crate::json_surface::append_project_plan_json_fields(&mut out, &plan);
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_usize_field("tests_declared", declared_tests.len()),
            crate::json_field(
                "artifact_output_dir",
                &artifact_output_dir.display().to_string(),
            ),
            crate::json_bool_field("artifact_ready_to_run", artifact_report.ready_to_run),
            crate::json_field(
                "artifact_recommended_next_step",
                &artifact_report.recommended_next_step,
            ),
            crate::json_field(
                "artifact_recommended_command",
                &artifact_report.recommended_command,
            ),
            crate::json_field(
                "artifact_nsld_drive_dry_run_command",
                &crate::workflow::nsld_drive_dry_run_command_for_output_dir(&artifact_output_dir),
            ),
            crate::json_field(
                "artifact_nsld_drive_dry_run_json_command",
                &crate::workflow::nsld_drive_dry_run_json_command_for_output_dir(
                    &artifact_output_dir,
                ),
            ),
            crate::json_field(
                "artifact_nsld_drive_apply_next_command",
                &crate::workflow::nsld_drive_apply_next_command_for_output_dir(
                    &artifact_output_dir,
                ),
            ),
            crate::json_field(
                "artifact_nsld_drive_apply_next_json_command",
                &crate::workflow::nsld_drive_apply_next_json_command_for_output_dir(
                    &artifact_output_dir,
                ),
            ),
            crate::json_field(
                "artifact_nsld_drive_apply_until_clean_command",
                &crate::workflow::nsld_drive_apply_until_clean_command_for_output_dir(
                    &artifact_output_dir,
                ),
            ),
            crate::json_field(
                "artifact_nsld_drive_apply_until_clean_json_command",
                &crate::workflow::nsld_drive_apply_until_clean_json_command_for_output_dir(
                    &artifact_output_dir,
                ),
            ),
            crate::workflow::nsld_drive_command_set_json_field(
                "artifact_nsld_drive_command_set",
                Some(&crate::workflow::nsld_drive_command_set_for_output_dir(
                    &artifact_output_dir,
                )),
            ),
        ],
    );
    append_json_field_strings(
        &mut out,
        artifact_report
            .payload_decoder_manifest
            .json_fields_with_prefix("artifact_payload_decoder_manifest"),
    );
    append_link_plan_json_fields(&mut out, link_plan.as_ref());
    crate::append_project_workflow_json_fields(&mut out, &frontdoor, include_galaxy_flow);
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_field(
                "abi_mode",
                if plan.abi_resolution.explicit {
                    "explicit"
                } else {
                    "auto-recommended"
                },
            ),
            crate::json_string_array_field(
                "galaxy_dependencies",
                &project
                    .manifest
                    .galaxy_dependencies
                    .iter()
                    .map(|item| format!("{}={}", item.name, item.version))
                    .collect::<Vec<_>>(),
            ),
            crate::json_usize_field("galaxy_surface_ids_count", galaxy_surface_ids.len()),
            crate::json_string_array_field("galaxy_surface_ids", &galaxy_surface_ids),
            crate::json_object_array_field("galaxy_records", &galaxy_records),
            crate::json_usize_field(
                "galaxy_imports_count",
                project.manifest.galaxy_imports.len(),
            ),
            crate::json_string_array_field(
                "galaxy_imports",
                &project
                    .manifest
                    .galaxy_imports
                    .iter()
                    .map(|item| format!("{}:{}", item.galaxy, item.library_module))
                    .collect::<Vec<_>>(),
            ),
            crate::json_usize_field(
                "galaxy_hidden_manual_only_library_modules_count",
                hidden_manual_only_library_modules.len(),
            ),
            crate::json_string_array_field(
                "galaxy_hidden_manual_only_library_modules",
                &hidden_manual_only_library_modules,
            ),
        ],
    );
    let lock_path = project.root.join("nuis.galaxy.lock");
    let declared_galaxy_dependencies = project
        .manifest
        .galaxy_dependencies
        .iter()
        .map(|item| format!("{}={}", item.name, item.version))
        .collect::<Vec<_>>();
    crate::json_surface::append_galaxy_lock_json_fields(
        &mut out,
        galaxy_lock_status,
        &lock_path,
        &declared_galaxy_dependencies,
    );
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_object_array_field("tests", &test_json),
            crate::json_object_array_field("public_surface_records", &public_surface_json),
        ],
    );
    out.push_str(",\"domains\":[");
    out.push_str(&domain_json);
    out.push_str("]}");
    Ok(out)
}
