use super::*;

pub(crate) fn render_project_doctor_json(input: &Path) -> Result<String, String> {
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
    crate::json_surface::append_public_surface_summary_json_fields(&mut out, &public_surface);
    crate::json_surface::append_project_plan_json_fields(&mut out, &plan);
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_usize_field("tests_declared", declared_tests.len()),
            crate::json_usize_field("tests_missing", missing_tests.len()),
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
    append_link_plan_json_fields(&mut out, link_plan.as_ref());
    crate::append_project_workflow_json_fields(&mut out, &frontdoor, include_galaxy_flow);
    append_json_field_strings(
        &mut out,
        vec![crate::json_field(
            "abi_mode",
            if plan.abi_resolution.explicit {
                "explicit"
            } else {
                "auto-recommended"
            },
        )],
    );
    crate::json_surface::append_project_check_summary_json_fields(
        &mut out,
        &abi_checks,
        &registry_checks,
        &lowering_checks,
    );
    append_json_field_strings(
        &mut out,
        vec![crate::json_field(
            "galaxy_manifest",
            &galaxy_manifest_display,
        )],
    );
    match galaxy_check {
        Some(Ok(checked)) => {
            append_json_field_strings(
                &mut out,
                vec![
                    crate::json_field("galaxy_check_status", "ok"),
                    crate::json_field("galaxy_package_kind", &checked.manifest.package_kind),
                    crate::json_field(
                        "galaxy_framework",
                        checked.manifest.framework.as_deref().unwrap_or("<none>"),
                    ),
                    crate::json_usize_field("galaxy_include_files", checked.include_files.len()),
                ],
            );
        }
        Some(Err(error)) => {
            append_json_field_strings(
                &mut out,
                vec![
                    crate::json_field("galaxy_check_status", "invalid"),
                    crate::json_field("galaxy_error", &error),
                ],
            );
        }
        None => {
            append_json_field_strings(
                &mut out,
                vec![crate::json_field("galaxy_check_status", "skipped")],
            );
        }
    }
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_field("galaxy_lock_status", &galaxy_doctor.lock_status),
            crate::json_field(
                "galaxy_lock_path",
                &galaxy_doctor.lock_path.display().to_string(),
            ),
        ],
    );
    if let Some(error) = galaxy_doctor.lock_error.as_deref() {
        append_json_field_strings(
            &mut out,
            vec![crate::json_field("galaxy_lock_error", error)],
        );
    }
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_field(
                "galaxy_deps_root",
                &galaxy_doctor.deps_root.display().to_string(),
            ),
            crate::json_field(
                "galaxy_local_registry",
                &galaxy_doctor.local_registry_root.display().to_string(),
            ),
            crate::json_usize_field(
                "galaxy_dependencies_count",
                galaxy_doctor.dependencies.len(),
            ),
            crate::json_usize_field(
                "galaxy_imports_count",
                project.manifest.galaxy_imports.len(),
            ),
            crate::json_usize_field("galaxy_surface_ids_count", galaxy_surface_ids.len()),
            crate::json_string_array_field("galaxy_surface_ids", &galaxy_surface_ids),
            crate::json_object_array_field("galaxy_records", &galaxy_records),
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
            crate::json_optional_string_field(
                "ns_nova_profile",
                nova_profile
                    .as_ref()
                    .map(|profile| profile.path.display().to_string())
                    .as_deref(),
            ),
            crate::json_optional_string_field(
                "ns_nova_stdlib_manifest",
                nova_stdlib
                    .as_ref()
                    .map(|summary| summary.path.display().to_string())
                    .as_deref(),
            ),
        ],
    );
    if let Some(error) = lock_error.as_deref() {
        append_json_field_strings(&mut out, vec![crate::json_field("note", error)]);
    }
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_string_array_field("next_steps", &next_steps),
            crate::json_object_array_field("tests", &tests_json),
            crate::json_object_array_field("public_surface_records", &public_surface_json),
            crate::json_object_array_field(
                "abi_checks",
                &crate::project_abi_checks_json(&abi_checks),
            ),
            crate::json_object_array_field(
                "registry_checks",
                &crate::project_domain_registry_checks_json(&registry_checks),
            ),
            crate::json_object_array_field(
                "lowering_checks",
                &crate::project_lowering_checks_json(&lowering_checks),
            ),
            crate::json_object_array_field("galaxy_dependencies", &dependency_json),
        ],
    );
    out.push_str(",\"domains\":[");
    out.push_str(&domain_json);
    out.push_str("]}");
    Ok(out)
}
