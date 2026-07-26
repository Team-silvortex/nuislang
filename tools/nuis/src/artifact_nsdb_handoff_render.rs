use crate::{
    artifact_launch_binding::FINAL_IMAGE_BINDING_PROOF_CONTRACT,
    artifact_launch_evidence::RunArtifactLaunchEvidence,
};

pub(crate) fn render_launch_evidence_nsdb_handoff(evidence: &RunArtifactLaunchEvidence) -> String {
    let records = evidence.payload_execution_trace_records();
    let ready_record_count = records
        .iter()
        .filter(|record| record.status == "ready")
        .count();
    let mut out = String::new();
    push_string(
        &mut out,
        "protocol",
        "nuis-nsdb-payload-execution-handoff-v1",
    );
    push_string(
        &mut out,
        "debugger_contract",
        evidence.payload_execution_trace_protocol(),
    );
    push_string(&mut out, "source", "run-artifact-launch-evidence");
    out.push_str(&format!("record_count = {}\n", records.len()));
    out.push_str(&format!("ready_record_count = {ready_record_count}\n"));
    render_final_image_binding_proof(&mut out, evidence);
    push_string(
        &mut out,
        "hetero_execution_closure_protocol",
        evidence.hetero_execution_closure_protocol(),
    );
    push_string(
        &mut out,
        "hetero_execution_closure_status",
        evidence.hetero_execution_closure_status(),
    );
    push_string(
        &mut out,
        "hetero_execution_closure_ready",
        if evidence.hetero_execution_closure_ready() {
            "true"
        } else {
            "false"
        },
    );
    push_optional_string(
        &mut out,
        "hetero_execution_closure_first_blocker",
        evidence.hetero_execution_closure_first_blocker(),
    );
    push_string(
        &mut out,
        "hetero_execution_closure_next_action",
        evidence.hetero_execution_closure_next_action(),
    );
    if let Some(first) = records.first() {
        push_string(&mut out, "first_trace_id", &first.trace_id);
        push_string(&mut out, "first_status", &first.status);
        push_string(&mut out, "first_next_action", &first.next_action);
    }
    for record in records {
        out.push_str("\n[[records]]\n");
        push_string(&mut out, "trace_id", &record.trace_id);
        push_string(&mut out, "status", &record.status);
        push_string(&mut out, "execution_phase", &record.execution_phase);
        push_optional_string(&mut out, "target", record.target.as_deref());
        push_optional_string(&mut out, "entry_symbol", record.entry_symbol.as_deref());
        push_optional_string(&mut out, "entry_kind", record.entry_kind.as_deref());
        push_optional_string(
            &mut out,
            "entry_section_id",
            record.entry_section_id.as_deref(),
        );
        push_optional_string(&mut out, "first_blocker", record.first_blocker.as_deref());
        push_string(&mut out, "next_action", &record.next_action);
    }
    out
}

fn render_final_image_binding_proof(out: &mut String, evidence: &RunArtifactLaunchEvidence) {
    let Some(proof) = evidence.final_image_binding_proof() else {
        return;
    };
    push_string(
        out,
        "final_image_binding_proof_contract",
        FINAL_IMAGE_BINDING_PROOF_CONTRACT,
    );
    out.push_str(&format!(
        "final_image_metadata_binding_count = {}\n",
        proof.binding_count
    ));
    push_string(
        out,
        "final_image_metadata_binding_table_hash",
        &proof.binding_table_hash,
    );
    push_string(
        out,
        "final_image_metadata_binding_validation_status",
        &proof.validation_status,
    );
    push_optional_string(
        out,
        "final_image_selected_provider_bundle_set_contract",
        proof.selected_set_contract.as_deref(),
    );
    match proof.selected_set_count {
        Some(count) => out.push_str(&format!(
            "final_image_selected_provider_bundle_count = {count}\n"
        )),
        None => out.push_str("final_image_selected_provider_bundle_count = 0\n"),
    }
    push_optional_string(
        out,
        "final_image_selected_provider_bundle_set_hash",
        proof.selected_set_hash.as_deref(),
    );
    push_string(out, "final_image_binding_proof_hash", &proof.proof_hash());
}

fn push_optional_string(out: &mut String, key: &str, value: Option<&str>) {
    push_string(out, key, value.unwrap_or(""));
}

fn push_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = \"");
    out.push_str(
        &value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n"),
    );
    out.push_str("\"\n");
}
