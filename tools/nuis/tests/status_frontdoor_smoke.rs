use std::{path::Path, process::Command};

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn status_prints_closure_then_tensor_handoff_sample() {
    let output = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("status")
        .current_dir(repo_root())
        .output()
        .expect("run nuis status");

    assert!(
        output.status.success(),
        "nuis status failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dev_tensor_hierarchy_protocol: nuis-dev-tensor-hierarchy-v1"));
    assert!(stdout.contains("dev_tensor_hierarchy_validation_status: clean"));
    assert!(stdout.contains("dev_tensor_hierarchy_validation_error_count: 0"));
    let reading_order = stdout
        .find("frontdoor_reading_order: closure_summary -> dev_tensor_weakest_task_card_handoff")
        .expect("status should expose frontdoor reading order");
    let closure_sample = stdout
        .find("frontdoor_sample_closure_summary: closure_summary_status -> closure_summary_next_action -> closure_summary_next_command")
        .expect("status should expose closure summary sample");
    let tensor_sample = stdout
        .find("frontdoor_sample_tensor_handoff: dev_tensor_weakest_task_card_coordinate -> dev_tensor_weakest_task_card_handoff_coordinate -> dev_tensor_weakest_task_card_handoff_command")
        .expect("status should expose tensor handoff sample");
    let tensor_handoff = stdout
        .find("dev_tensor_weakest_task_card_handoff_coordinate:")
        .expect("status should expose tensor handoff coordinate");

    assert!(
        reading_order < closure_sample
            && closure_sample < tensor_sample
            && tensor_sample < tensor_handoff,
        "status should present closure-summary guidance before tensor handoff details\n{stdout}"
    );
}
