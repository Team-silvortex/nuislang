use super::*;

pub fn build_artifact_lowering_alignment_summary(
    artifact: &LinkPlanArtifact,
    domain_units: &[LinkPlanDomainUnit],
) -> ArtifactLoweringAlignmentSummary {
    let artifact_units = &artifact.lowering_units;
    if artifact_units.is_empty() && artifact.section_count.unwrap_or(0) == 0 {
        return ArtifactLoweringAlignmentSummary {
            checked: 0,
            mismatches: 0,
            consistent: true,
            checks: Vec::new(),
        };
    }
    let checked = artifact_units.len();
    let mut checks = artifact_units
        .iter()
        .enumerate()
        .map(|(index, artifact_unit)| {
            let mut issues = Vec::new();
            let Some(domain_unit) = domain_units.get(index) else {
                issues.push("missing_manifest_domain_unit".to_owned());
                return ArtifactLoweringAlignmentCheck {
                    index,
                    package_id: artifact_unit.package_id.clone(),
                    domain_family: artifact_unit.domain_family.clone(),
                    consistent: false,
                    issues,
                };
            };
            push_alignment_issue(
                &mut issues,
                "package_id",
                Some(artifact_unit.package_id.as_str()),
                Some(domain_unit.package_id.as_str()),
            );
            push_alignment_issue(
                &mut issues,
                "domain_family",
                Some(artifact_unit.domain_family.as_str()),
                Some(domain_unit.domain_family.as_str()),
            );
            push_alignment_issue(
                &mut issues,
                "backend_family",
                artifact_unit.backend_family.as_deref(),
                domain_unit.backend_family.as_deref(),
            );
            push_alignment_issue(
                &mut issues,
                "selected_lowering_target",
                artifact_unit.selected_lowering_target.as_deref(),
                domain_unit.selected_lowering_target.as_deref(),
            );
            push_alignment_issue(
                &mut issues,
                "artifact_ir_sidecar_path",
                artifact_unit.artifact_ir_sidecar_path.as_deref(),
                domain_unit.artifact_ir_sidecar_path.as_deref(),
            );
            push_alignment_issue(
                &mut issues,
                "contract_family",
                Some(artifact_unit.contract_family.as_str()),
                Some(domain_unit.contract_family.as_str()),
            );
            push_alignment_issue(
                &mut issues,
                "packaging_role",
                Some(artifact_unit.packaging_role.as_str()),
                Some(domain_unit.packaging_role.as_str()),
            );
            ArtifactLoweringAlignmentCheck {
                index,
                package_id: artifact_unit.package_id.clone(),
                domain_family: artifact_unit.domain_family.clone(),
                consistent: issues.is_empty(),
                issues,
            }
        })
        .collect::<Vec<_>>();
    if artifact_units.len() < domain_units.len() {
        for index in artifact_units.len()..domain_units.len() {
            let domain_unit = &domain_units[index];
            checks.push(ArtifactLoweringAlignmentCheck {
                index,
                package_id: domain_unit.package_id.clone(),
                domain_family: domain_unit.domain_family.clone(),
                consistent: false,
                issues: vec!["missing_artifact_lowering_unit".to_owned()],
            });
        }
    }
    let mismatches = checks.iter().filter(|check| !check.consistent).count();
    ArtifactLoweringAlignmentSummary {
        checked,
        mismatches,
        consistent: mismatches == 0,
        checks,
    }
}

fn push_alignment_issue(
    issues: &mut Vec<String>,
    field: &str,
    artifact_value: Option<&str>,
    domain_value: Option<&str>,
) {
    if artifact_value == domain_value {
        return;
    }
    issues.push(format!(
        "{field}:artifact={}:manifest={}",
        artifact_value.unwrap_or("<none>"),
        domain_value.unwrap_or("<none>")
    ));
}
