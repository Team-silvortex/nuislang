use super::*;

pub fn validate_project_lowering_selections(
    resolution: &ProjectAbiResolution,
) -> Vec<ProjectLoweringSelectionView> {
    let mut views = resolution
        .requirements
        .iter()
        .map(|item| {
            let mut issues = Vec::new();
            let mut registered_lowering_targets = Vec::new();
            let mut selected_lowering_target = None;
            match crate::registry::load_domain_registration_for_domain(
                Path::new("nustar-packages"),
                &item.domain,
            ) {
                Ok(registration) => {
                    registered_lowering_targets = registration.lowering_targets.clone();
                    if registered_lowering_targets.is_empty() {
                        issues.push(ProjectLoweringIssue {
                            kind: ProjectLoweringIssueKind::NoRegisteredLoweringTargets,
                            message: format!(
                                "registered domain `{}` does not declare any lowering targets",
                                item.domain
                            ),
                        });
                    }
                    match selected_lowering_target_for_domain(
                        &item.domain,
                        &item.abi,
                        &registered_lowering_targets,
                    ) {
                        Ok(selected) => {
                            selected_lowering_target = selected;
                        }
                        Err(error) => issues.push(ProjectLoweringIssue {
                            kind: ProjectLoweringIssueKind::AbiTargetResolutionFailed,
                            message: error,
                        }),
                    }
                    if let Some(selected) = selected_lowering_target.as_deref() {
                        if !registered_lowering_targets
                            .iter()
                            .any(|target| target == selected)
                        {
                            issues.push(ProjectLoweringIssue {
                                kind: ProjectLoweringIssueKind::SelectedLoweringTargetNotRegistered,
                                message: format!(
                                    "selected lowering target `{selected}` is not declared by registered lowering targets: {}",
                                    if registered_lowering_targets.is_empty() {
                                        "<none>".to_owned()
                                    } else {
                                        registered_lowering_targets.join(", ")
                                    }
                                ),
                            });
                        }
                    }
                }
                Err(error) => issues.push(ProjectLoweringIssue {
                    kind: ProjectLoweringIssueKind::DomainNotRegistered,
                    message: error,
                }),
            }
            ProjectLoweringSelectionView {
                domain: item.domain.clone(),
                abi: Some(item.abi.clone()),
                registered_lowering_targets,
                selected_lowering_target,
                ok: issues.is_empty(),
                issues,
            }
        })
        .collect::<Vec<_>>();
    views.sort_by(|lhs, rhs| lhs.domain.cmp(&rhs.domain));
    views
}

pub fn render_project_lowering_selection_lines(view: &ProjectLoweringSelectionView) -> Vec<String> {
    let mut out = String::new();
    write_project_lowering_selection_lines(&mut out, view)
        .expect("writing project lowering selection lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn write_project_lowering_selection_lines<W: fmt::Write>(
    out: &mut W,
    view: &ProjectLoweringSelectionView,
) -> fmt::Result {
    write!(
        out,
        "lowering: {} abi={} ok={} selected={} registered=",
        view.domain,
        view.abi.as_deref().unwrap_or("<none>"),
        if view.ok { "yes" } else { "no" },
        view.selected_lowering_target.as_deref().unwrap_or("<none>"),
    )?;
    if view.registered_lowering_targets.is_empty() {
        out.write_str("<none>")?;
    } else {
        write_joined(
            out,
            &view.registered_lowering_targets,
            ", ",
            |out, target| write!(out, "{target}"),
        )?;
    }
    writeln!(out, "\tissues={}", view.issue_count())?;
    for issue in &view.issues {
        writeln!(
            out,
            "lowering_issue: {}",
            issue.summary().replace(": ", " ")
        )?;
    }
    Ok(())
}

pub fn project_lowering_issue_json(issue: &ProjectLoweringIssue) -> String {
    format!(
        "{{\"code\":\"{}\",\"kind\":\"{}\",\"message\":\"{}\"}}",
        issue.kind.code(),
        issue.kind.as_str(),
        json_escape(&issue.message)
    )
}

pub fn project_lowering_selection_json(view: &ProjectLoweringSelectionView) -> String {
    let issues = view
        .issues
        .iter()
        .map(project_lowering_issue_json)
        .collect::<Vec<_>>()
        .join(",");
    let registered = view
        .registered_lowering_targets
        .iter()
        .map(|target| format!("\"{}\"", json_escape(target)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"domain\":\"{}\",\"abi\":{},\"registered_lowering_targets\":[{}],\"selected_lowering_target\":{},\"ok\":{},\"issues\":[{}]}}",
        json_escape(&view.domain),
        view.abi
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        registered,
        view.selected_lowering_target
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        if view.ok { "true" } else { "false" },
        issues
    )
}

pub fn ensure_project_lowering_selections_valid(
    resolution: &ProjectAbiResolution,
) -> Result<(), String> {
    let failures = validate_project_lowering_selections(resolution)
        .into_iter()
        .filter(|view| !view.ok)
        .map(|view| view.summary_line())
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "project lowering selection validation failed:\n{}",
            failures.join("\n")
        ))
    }
}
