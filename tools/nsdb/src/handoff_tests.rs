use super::read_payload_execution_handoff;
use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn reads_ready_payload_execution_handoff() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir: PathBuf = env::temp_dir().join(format!("nsdb-handoff-{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("nuis.nsdb.payload-execution-handoff.toml"),
        r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
record_count = 2
ready_record_count = 1
hetero_execution_closure_protocol = "nuis-hetero-execution-closure-v1"
hetero_execution_closure_status = "closed"
hetero_execution_closure_ready = "true"
hetero_execution_closure_next_action = "handoff-hetero-execution-evidence-to-nsdb"
first_trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
first_status = "ready"
first_next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
status = "ready"
execution_phase = "container-loader-handoff"
entry_symbol = "nuis.bootstrap.lifecycle.v1"
next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:shader:pixelmagic.blur"
status = "blocked"
execution_phase = "device-dispatch"
target = "shader"
entry_symbol = "pixelmagic.blur"
entry_kind = "shader-kernel"
entry_section_id = "sec0002.shader"
first_blocker = "device-execution-sample-missing"
next_action = "materialize-device-execution-trace"
"#,
    )
    .unwrap();

    let handoff = read_payload_execution_handoff(&dir);

    assert!(handoff.available);
    assert_eq!(handoff.status, "ready");
    assert_eq!(
        handoff.final_image_binding_proof.proof_status,
        "legacy-unbound"
    );
    assert_eq!(handoff.record_count, 2);
    assert_eq!(handoff.events.len(), 2);
    assert_eq!(handoff.events[0].index, 0);
    assert_eq!(handoff.events[0].trace_id, handoff.first_trace_id);
    assert_eq!(
        handoff.events[0].next_action,
        "handoff-payload-trace-to-nsdb"
    );
    assert_eq!(handoff.events[1].index, 1);
    assert_eq!(handoff.events[1].status, "blocked");
    assert_eq!(
        handoff.events[1].first_blocker,
        "device-execution-sample-missing"
    );
    assert_eq!(handoff.first_execution_phase, "container-loader-handoff");
    assert_eq!(handoff.first_entry_symbol, "nuis.bootstrap.lifecycle.v1");
    assert_eq!(
        handoff.hetero_execution_closure_protocol,
        "nuis-hetero-execution-closure-v1"
    );
    assert_eq!(handoff.hetero_execution_closure_status, "closed");
    assert_eq!(handoff.hetero_execution_closure_ready, "true");
    assert_eq!(
        handoff.hetero_execution_closure_next_action,
        "handoff-hetero-execution-evidence-to-nsdb"
    );
}
