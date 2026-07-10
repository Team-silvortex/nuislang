use super::{display_text::*, reports::*};

pub(crate) fn print_nsld_final_stage_plan_report(report: &NsldFinalStagePlanReport) {
    println!("Nsld final-stage plan");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  plan_hash: {}", report.plan_hash);
    println!("  final_stage_kind: {}", report.final_stage_kind);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  final_output_path: {}", report.final_output_path);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  compatibility_mode: {}", report.compatibility_mode);
    println!("  input_count: {}", report.input_count);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_hash: {}", report.payload_hash);
    println!("  linker_contract_hash: {}", report.linker_contract_hash);
    println!(
        "  native_object_required: {}",
        report.native_object_required
    );
    println!("  native_object_present: {}", report.native_object_present);
    for input in &report.inputs {
        println!(
            "  final_stage_input: order={} id={} kind={} required={} present={} hash={} path={}",
            input.order_index,
            input.input_id,
            input.input_kind,
            input.required,
            input.present,
            input.content_hash,
            input.path
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_stage_plan_emit_report(report: &NsldFinalStagePlanEmitReport) {
    println!("Nsld final-stage plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  plan_hash: {}", report.plan_hash);
    println!("  input_count: {}", report.input_count);
    println!("  blocker_count: {}", report.blocker_count);
}

pub(crate) fn print_nsld_final_stage_plan_verify_report(report: &NsldFinalStagePlanVerifyReport) {
    println!("Nsld final-stage plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_plan_hash: {}", report.expected_plan_hash);
    println!(
        "  actual_plan_hash: {}",
        optional_string_text(report.actual_plan_hash.as_deref())
    );
    println!("  expected_input_count: {}", report.expected_input_count);
    println!(
        "  actual_input_count: {}",
        optional_usize_text(report.actual_input_count)
    );
    for input_id in &report.expected_input_ids {
        println!("  expected_input_id: {input_id}");
    }
    for input_id in &report.actual_input_ids {
        println!("  actual_input_id: {input_id}");
    }
    println!(
        "  expected_input_entry_count: {}",
        report.expected_input_entry_count
    );
    println!(
        "  actual_input_entry_count: {}",
        report.actual_input_entry_count
    );
    for blocker in &report.expected_blockers {
        println!("  expected_blocker: {blocker}");
    }
    for blocker in &report.actual_blockers {
        println!("  actual_blocker: {blocker}");
    }
    for note in &report.expected_notes {
        println!("  expected_note: {note}");
    }
    for note in &report.actual_notes {
        println!("  actual_note: {note}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
