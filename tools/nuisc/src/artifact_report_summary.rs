use super::*;

pub(crate) fn artifact_report_summary_lines(
    artifact_verify: &aot::NuisCompiledArtifactVerifyReport,
    verification_summary: &DomainBuildVerificationSummary,
    link_plan: Option<&linker::LinkPlan>,
    manifest_verify_reconstructed: bool,
    execution_overview: Option<&ExecutionInspectOverview>,
    doc_indexes: Option<&[frontend::AstDocIndex]>,
    project_metadata: Option<&ProjectMetadataSummary>,
) -> Vec<String> {
    let mut lines = vec![
        format!(
            "summary: artifact_roundtrip={} lifecycle={} runtime_flags={} all_units_consistent={}",
            if artifact_verify.artifact_roundtrip_verified {
                "ok"
            } else {
                "failed"
            },
            if artifact_verify.lifecycle_contract_consistent {
                "ok"
            } else {
                "failed"
            },
            if artifact_verify.lifecycle_runtime_capability_flags_consistent {
                "ok"
            } else {
                "failed"
            },
            if verification_summary.all_units_consistent {
                "true"
            } else {
                "false"
            }
        ),
        format!(
            "summary_units: total={} host={} hetero={} drift={} failing={}",
            verification_summary.total_units,
            verification_summary.host_units_checked,
            verification_summary.hetero_units_checked,
            verification_summary.registry_drift_units,
            if verification_summary.failing_units.is_empty() {
                "<none>".to_owned()
            } else {
                verification_summary.failing_units.join(", ")
            }
        ),
        format!(
            "summary_manifest: reconstructed={}",
            if manifest_verify_reconstructed {
                "true"
            } else {
                "false"
            }
        ),
    ];
    if let Some(plan) = link_plan {
        lines.push(format!(
            "summary_link: final_stage={} driver={} link_mode={} output={}",
            plan.final_stage.kind,
            plan.final_stage.driver,
            plan.final_stage.link_mode,
            plan.final_stage.output_path
        ));
    }
    if let Some(overview) = execution_overview {
        let issues = execution_inspect_issues(overview);
        lines.push(format!(
            "summary_execution: hetero_domains={} domains={}",
            overview.heterogeneous_domains,
            if overview.domains.is_empty() {
                "<none>".to_owned()
            } else {
                overview
                    .domains
                    .iter()
                    .map(|domain| {
                        let target = domain
                            .selected_lowering_target
                            .as_deref()
                            .unwrap_or("<none>");
                        format!(
                            "{}(target={} phases={} events={})",
                            domain.domain_family, target, domain.phase_count, domain.event_count
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
        lines.push(format!(
            "summary_execution_issues: {}",
            if issues.is_empty() {
                "<none>".to_owned()
            } else {
                issues
                    .iter()
                    .map(|issue| format!("{}:{}", issue.domain_family, issue.issue))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
    }
    if let Some(indexes) = doc_indexes {
        let module_count = indexes.len();
        let item_count = indexes.iter().map(|index| index.items.len()).sum::<usize>();
        lines.push(format!(
            "summary_docs: modules={} documented_items={} documented_modules={}",
            module_count,
            item_count,
            if indexes.is_empty() {
                "<none>".to_owned()
            } else {
                indexes
                    .iter()
                    .map(|index| index.module_path.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
    }
    if let Some(project) = project_metadata {
        lines.push(format!(
            "summary_project: docs={}/{}/{} imports={}/{}/{}/{}/{} galaxies={}/{}/{}/{}",
            project.docs_module_count,
            project.docs_documented_module_count,
            project.docs_documented_item_count,
            project.imports_library_count,
            project.imports_visible_library_count,
            project.imports_visible_module_count,
            project.imports_documented_visible_module_count,
            project.imports_documented_visible_item_count,
            project.galaxy_count,
            project.documented_galaxy_count,
            project.documented_galaxy_library_module_count,
            project.documented_galaxy_item_count
        ));
    }
    lines
}
