use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain_build_report::{
    collect_domain_build_unit_verdicts, domain_build_contract_drift_checks,
    domain_build_contract_drift_json, domain_build_unit_contracts_json,
    domain_build_unit_verification_verdict_json, domain_build_verification_summary_json,
    summarize_domain_build_verification, DomainBuildVerificationSummary,
};
use crate::execution_inspect::{execution_inspect_issues, ExecutionInspectOverview};
use crate::execution_inspect_report::inspect_execution_json;
use crate::inspect_report::{collect_doc_indexes_from_manifest_input, inspect_docs_json};
use crate::json_report::{
    artifact_lowering_units_json, json_bool_field, json_optional_string_field,
    json_string_array_field, json_string_field, json_usize_field,
};
use crate::link_report::link_plan_json;
use crate::project_metadata_report::{
    inspect_project_metadata_json, project_metadata_summary_from_manifest_report,
    ProjectMetadataSummary,
};
use crate::{aot, frontend, linker, registry};

#[path = "artifact_report_domain.rs"]
mod artifact_report_domain;
#[path = "artifact_report_inspect.rs"]
mod artifact_report_inspect;
#[path = "artifact_report_json.rs"]
mod artifact_report_json_impl;
#[path = "artifact_report_reconstruct.rs"]
mod artifact_report_reconstruct;
#[path = "artifact_report_summary.rs"]
mod artifact_report_summary;
#[path = "artifact_report_verify.rs"]
mod artifact_report_verify;

#[allow(unused_imports)]
pub(crate) use artifact_report_domain::{
    domain_build_contract_summary_json, domain_build_unit_json, domain_registry_json,
};
pub(crate) use artifact_report_inspect::inspect_artifact_json;
pub(crate) use artifact_report_json_impl::artifact_report_json;
pub(crate) use artifact_report_reconstruct::reconstruct_manifest_report_from_artifact;
pub(crate) use artifact_report_summary::artifact_report_summary_lines;
pub(crate) use artifact_report_verify::{verify_artifact_json, verify_build_manifest_json};
