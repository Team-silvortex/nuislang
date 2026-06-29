#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionInspectDomainOverview {
    pub(crate) domain_family: String,
    pub(crate) selected_lowering_target: Option<String>,
    pub(crate) phase_count: usize,
    pub(crate) event_count: usize,
    pub(crate) resource_keys: Vec<String>,
    pub(crate) output_handles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionInspectOverview {
    pub(crate) heterogeneous_domains: usize,
    pub(crate) domains: Vec<ExecutionInspectDomainOverview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExecutionInspectIssue {
    pub(crate) domain_family: String,
    pub(crate) issue: String,
}

pub(crate) fn execution_inspect_issues(
    overview: &ExecutionInspectOverview,
) -> Vec<ExecutionInspectIssue> {
    let mut issues = Vec::new();
    for domain in &overview.domains {
        if domain.selected_lowering_target.is_none() {
            issues.push(ExecutionInspectIssue {
                domain_family: domain.domain_family.clone(),
                issue: "missing_target".to_owned(),
            });
        }
        if domain.phase_count == 0 {
            issues.push(ExecutionInspectIssue {
                domain_family: domain.domain_family.clone(),
                issue: "zero_phases".to_owned(),
            });
        }
        if domain.phase_count != domain.event_count {
            issues.push(ExecutionInspectIssue {
                domain_family: domain.domain_family.clone(),
                issue: format!(
                    "phase_event_mismatch({}->{})",
                    domain.phase_count, domain.event_count
                ),
            });
        }
        let has_resource = |key: &str| domain.resource_keys.iter().any(|item| item == key);
        let has_output = |key: &str| domain.output_handles.iter().any(|item| item == key);
        match domain.domain_family.as_str() {
            "network" => {
                if !has_resource("request_packet") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_network_request_packet".to_owned(),
                    });
                }
                if !has_resource("active_response") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_network_active_response".to_owned(),
                    });
                }
                if !has_output("response.handle") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_network_response_handle".to_owned(),
                    });
                }
            }
            "shader" => {
                if !has_resource("shader_buffer") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_shader_buffer".to_owned(),
                    });
                }
                if !has_resource("frame_target") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_shader_frame_target".to_owned(),
                    });
                }
                if !has_output("draw.handle") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_shader_draw_handle".to_owned(),
                    });
                }
            }
            "kernel" => {
                if !has_resource("kernel_buffer") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_kernel_buffer".to_owned(),
                    });
                }
                if !has_resource("dispatch_handle") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_kernel_dispatch_handle".to_owned(),
                    });
                }
                if !has_resource("result_buffer") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_kernel_result_buffer".to_owned(),
                    });
                }
            }
            _ => {}
        }
    }
    issues
}

pub(crate) fn verdict_status(ok: bool, hetero_expected: bool) -> &'static str {
    if !hetero_expected {
        "skipped"
    } else if ok {
        "ok"
    } else {
        "missing"
    }
}
