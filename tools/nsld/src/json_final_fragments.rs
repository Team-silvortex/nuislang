use super::{json_fields::*, reports::NsldFinalStageInputDiagnostic};

pub(crate) fn final_stage_inputs_json(inputs: &[NsldFinalStageInputDiagnostic]) -> String {
    inputs
        .iter()
        .map(|input| {
            let fields = [
                json_usize_field("order_index", input.order_index),
                json_string_field("input_id", &input.input_id),
                json_string_field("input_kind", &input.input_kind),
                json_string_field("path", &input.path),
                json_string_field("content_hash", &input.content_hash),
                json_bool_field("required", input.required),
                json_bool_field("present", input.present),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}
