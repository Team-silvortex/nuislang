use std::fmt;
use std::path::Path;

mod link_plan;
mod link_plan_readiness;
mod link_plan_text;
mod project_doctor;
mod project_doctor_json;
mod project_status;
mod scheduler;

use link_plan::{
    append_json_object_strings, append_link_plan_json_fields, load_link_plan,
    write_link_plan_text_fields,
};
#[cfg(test)]
pub(crate) use project_doctor::render_project_doctor_text_summary;
pub(crate) use project_doctor::write_project_doctor_text_summary;
pub(crate) use project_doctor_json::render_project_doctor_json;
#[cfg(test)]
pub(crate) use project_status::render_project_status_text_summary;
pub(crate) use project_status::{render_project_status_json, write_project_status_text_summary};
pub(crate) use scheduler::render_scheduler_view_json;

pub(crate) fn append_json_field_strings(
    out: &mut String,
    fields: impl IntoIterator<Item = String>,
) {
    for field in fields {
        if !out.ends_with('{') {
            out.push(',');
        }
        out.push_str(&field);
    }
}
