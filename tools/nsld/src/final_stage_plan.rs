use super::{fnv1a64_hex, reports::NsldFinalStageInputDiagnostic};
use std::path::PathBuf;

pub(crate) fn final_stage_input(
    order_index: usize,
    input_id: &str,
    input_kind: &str,
    path: PathBuf,
    required: bool,
) -> NsldFinalStageInputDiagnostic {
    let present = path.exists();
    let content_hash = if present {
        std::fs::read(&path)
            .map(|bytes| fnv1a64_hex(&bytes))
            .unwrap_or_else(|_| "missing".to_owned())
    } else {
        "missing".to_owned()
    };
    NsldFinalStageInputDiagnostic {
        order_index,
        input_id: input_id.to_owned(),
        input_kind: input_kind.to_owned(),
        path: path.display().to_string(),
        content_hash,
        required,
        present,
    }
}

pub(crate) fn final_stage_notes(
    plan: &nuisc::linker::LinkPlan,
    host_wrapper_required: bool,
) -> Vec<String> {
    let mut notes = Vec::new();
    if host_wrapper_required {
        notes.push(format!(
            "host-final-stage-driver:{}",
            plan.final_stage.driver
        ));
    }
    if !plan.cpu_target.object_format.is_empty() {
        notes.push(format!("object-format:{}", plan.cpu_target.object_format));
    }
    if !plan.cpu_target.clang_target.is_empty() {
        notes.push(format!("clang-target:{}", plan.cpu_target.clang_target));
    }
    notes
}

pub(crate) fn nsld_final_stage_plan_hash(
    plan: &nuisc::linker::LinkPlan,
    inputs: &[NsldFinalStageInputDiagnostic],
    container_hash: &str,
    payload_hash: &str,
    linker_contract_hash: &str,
    blockers: &[String],
    notes: &[String],
) -> String {
    let mut material = String::new();
    material.push_str(&plan.final_stage.kind);
    material.push('\t');
    material.push_str(&plan.final_stage.driver);
    material.push('\t');
    material.push_str(&plan.final_stage.link_mode);
    material.push('\t');
    material.push_str(&plan.final_stage.output_path);
    material.push('\n');
    material.push_str(container_hash);
    material.push('\t');
    material.push_str(payload_hash);
    material.push('\t');
    material.push_str(linker_contract_hash);
    material.push('\n');
    for input in inputs {
        material.push_str(&input.input_id);
        material.push('\t');
        material.push_str(&input.input_kind);
        material.push('\t');
        material.push_str(&input.path);
        material.push('\t');
        material.push_str(&input.content_hash);
        material.push('\t');
        material.push_str(if input.required {
            "required"
        } else {
            "optional"
        });
        material.push('\t');
        material.push_str(if input.present { "present" } else { "missing" });
        material.push('\n');
    }
    for blocker in blockers {
        material.push_str("blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    for note in notes {
        material.push_str("note\t");
        material.push_str(note);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}
