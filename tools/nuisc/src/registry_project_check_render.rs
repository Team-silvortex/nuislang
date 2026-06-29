use std::fmt;

use crate::registry::{ProjectDomainRegistryCheck, ProjectDomainRegistryIssue};
use crate::registry_json::{json_bool_field, json_field, json_optional_string_field};

pub fn project_domain_registry_issue_json(issue: &ProjectDomainRegistryIssue) -> String {
    format!(
        "{{{},{},{}}}",
        json_field("code", issue.kind.code()),
        json_field("kind", issue.kind.as_str()),
        json_field("message", &issue.message)
    )
}

pub fn project_domain_registry_check_json(check: &ProjectDomainRegistryCheck) -> String {
    let issue_json = check
        .issues
        .iter()
        .map(project_domain_registry_issue_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{},{},{},{},{},{},{}}}",
        json_field("domain", &check.domain),
        json_optional_string_field("package", check.package.as_deref()),
        json_optional_string_field("contract_schema", check.contract_schema.as_deref()),
        json_optional_string_field("abi", check.abi.as_deref()),
        json_bool_field("abi_registered", check.abi_registered),
        json_bool_field("ok", check.ok),
        format!("\"issues\":[{}]", issue_json)
    )
}

pub fn render_project_domain_registry_check_lines(
    check: &ProjectDomainRegistryCheck,
) -> Vec<String> {
    let mut out = String::new();
    write_project_domain_registry_check_lines(&mut out, check)
        .expect("writing project domain registry check lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn write_project_domain_registry_check_lines<W: fmt::Write>(
    out: &mut W,
    check: &ProjectDomainRegistryCheck,
) -> fmt::Result {
    writeln!(
        out,
        "registry: {} package={} schema={} abi={} ok={} abi_registered={} issues={}",
        check.domain,
        check.package.as_deref().unwrap_or("<missing>"),
        check.contract_schema.as_deref().unwrap_or("<missing>"),
        check.abi.as_deref().unwrap_or("<none>"),
        if check.ok { "yes" } else { "no" },
        if check.abi_registered { "yes" } else { "no" },
        check.issue_count()
    )?;
    for issue in &check.issues {
        writeln!(
            out,
            "registry_issue: {} {} {}",
            issue.kind.code(),
            issue.kind.as_str(),
            issue.message
        )?;
    }
    Ok(())
}
