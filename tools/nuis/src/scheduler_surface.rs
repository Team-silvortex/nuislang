use crate::{append_json_object_fields, WorkflowFrontdoorSurface};

#[derive(Debug, Clone)]
pub(crate) struct SchedulerViewDomainRecord {
    shared_domain_json: String,
    shared_abi_json: Option<String>,
}

pub(crate) fn project_plan_domains_json(
    plan: &nuisc::project::ProjectCompilationPlan,
) -> Result<String, String> {
    let mut domains = Vec::new();
    for item in &plan.abi_resolution.requirements {
        domains.push(scheduler_view_domain_record(
            &item.domain,
            None,
            Some(item.abi.clone()),
        )?);
    }
    Ok(domains
        .iter()
        .map(scheduler_view_domain_record_json)
        .collect::<Vec<_>>()
        .join(","))
}

#[allow(dead_code)]
pub(crate) fn project_workflow_json_fields(
    frontdoor: &WorkflowFrontdoorSurface,
    include_galaxy_flow: bool,
) -> Vec<String> {
    crate::json_surface::workflow_contract_json_fields(
        frontdoor,
        true,
        true,
        include_galaxy_flow,
        false,
    )
}

pub(crate) fn append_project_workflow_json_fields(
    out: &mut String,
    frontdoor: &WorkflowFrontdoorSurface,
    include_galaxy_flow: bool,
) {
    crate::json_surface::append_workflow_contract_json_fields(
        out,
        frontdoor,
        true,
        true,
        include_galaxy_flow,
        false,
    );
}

pub(crate) fn scheduler_view_domain_record(
    domain: &str,
    _package: Option<String>,
    abi: Option<String>,
) -> Result<SchedulerViewDomainRecord, String> {
    let registration = nuisc::registry::load_domain_registration_for_domain(
        std::path::Path::new("nustar-packages"),
        domain,
    )?;
    let shared_abi_json = if let Some(abi_name) = abi.as_deref() {
        let resolution = nuisc::project::ProjectAbiResolution {
            requirements: vec![nuisc::project::ProjectAbiRequirement {
                domain: domain.to_owned(),
                abi: abi_name.to_owned(),
            }],
            explicit: true,
        };
        nuisc::project::project_abi_selection_views(&resolution)
            .into_iter()
            .next()
            .map(|view| nuisc::project::project_abi_selection_view_json(&view))
    } else {
        None
    };
    Ok(SchedulerViewDomainRecord {
        shared_domain_json: nuisc::registry::domain_registration_json(&registration),
        shared_abi_json,
    })
}

pub(crate) fn scheduler_view_domain_record_json(record: &SchedulerViewDomainRecord) -> String {
    let mut fields = Vec::new();
    if let Some(shared_abi_json) = record.shared_abi_json.as_deref() {
        fields.push(format!("\"abi_selection\":{}", shared_abi_json));
    } else {
        fields.push("\"abi_selection\":null".to_owned());
    }
    append_json_object_fields(&record.shared_domain_json, &fields)
}
