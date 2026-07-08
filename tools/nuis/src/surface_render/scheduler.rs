use super::*;

pub(crate) fn render_scheduler_view_json(input: &Path) -> Result<String, String> {
    if nuisc::project::is_project_input(input) {
        let project = nuisc::project::load_project(input)?;
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
        let mut out = String::from("{");
        append_json_field_strings(
            &mut out,
            vec![
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
            ],
        );
        crate::json_surface::append_workflow_contract_json_fields(
            &mut out, &frontdoor, false, false, false, false,
        );
        crate::json_surface::append_project_plan_json_fields(&mut out, &plan);
        out.push_str(",\"domains\":[");
        append_json_object_strings(
            &mut out,
            &domains
                .iter()
                .map(crate::scheduler_view_domain_record_json)
                .collect::<Vec<_>>(),
        );
        out.push_str("]}");
        return Ok(out);
    }

    let artifacts = nuisc::pipeline::compile_source_path(input)?;
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
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            crate::json_field("source_kind", "single-file"),
            crate::json_field("input", &input.display().to_string()),
            crate::json_field("ast_domain", &artifacts.ast.domain),
            crate::json_field("ast_unit", &artifacts.ast.unit),
            crate::workflow_frontdoor_json_object_field(&frontdoor),
            crate::json_field("workflow_kind", frontdoor.workflow_kind),
            crate::json_field("workflow_brief", frontdoor.workflow_brief),
            crate::json_field("workflow_samples", frontdoor.workflow_samples),
            crate::json_field("recommended_next_step", frontdoor.recommended_next_step),
            crate::json_field("recommended_command", frontdoor.recommended_command),
            crate::json_field("recommended_reason", frontdoor.recommended_reason),
        ],
    );
    out.push_str(",\"domains\":[");
    append_json_object_strings(
        &mut out,
        &domains
            .iter()
            .map(crate::scheduler_view_domain_record_json)
            .collect::<Vec<_>>(),
    );
    out.push_str("]}");
    Ok(out)
}
