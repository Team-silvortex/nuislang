use super::*;
use yir_core::{Operation, ResourceKind};

fn cpu_resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn node(name: &str, instruction: &str, args: &[&str]) -> Node {
    Node {
        name: name.to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            &format!("cpu.{instruction}"),
            args.iter().map(|value| (*value).to_owned()).collect(),
        )
        .expect("CPU operation"),
    }
}

fn execute(
    cpu: &CpuMod,
    state: &mut ExecutionState,
    name: &str,
    instruction: &str,
    args: &[&str],
) -> Result<Value, String> {
    cpu.execute(&node(name, instruction, args), &cpu_resource(), state)
}

fn shared_mutex(cpu: &CpuMod, state: &mut ExecutionState, prefix: &str) {
    let value_name = format!("{prefix}_value");
    let mutex_name = format!("{prefix}_mutex");
    let shared_name = format!("{prefix}_shared");
    state.bind_value(&value_name, Value::Int(23));
    let mutex =
        execute(cpu, state, &mutex_name, "mutex_new", &[&value_name]).expect("create mutex");
    state.bind_value(&mutex_name, mutex);
    let shared =
        execute(cpu, state, &shared_name, "mutex_share", &[&mutex_name]).expect("share mutex");
    state.bind_value(&shared_name, shared);
}

#[test]
fn shared_close_revokes_permits_and_rejects_active_leases() {
    let cpu = CpuMod;
    let mut state = ExecutionState::default();
    state.bind_value("lane0", Value::Int(0));
    state.bind_value("lane1", Value::Int(1));

    shared_mutex(&cpu, &mut state, "revoked");
    let permit = execute(
        &cpu,
        &mut state,
        "pending",
        "mutex_permit",
        &["revoked_shared", "lane0"],
    )
    .expect("issue pending permit");
    state.bind_value("pending", permit);
    let revoked = execute(
        &cpu,
        &mut state,
        "closed",
        "mutex_shared_close",
        &["revoked_shared"],
    )
    .expect("close shared mutex");
    assert_eq!(revoked, Value::Int(1));
    let stale = execute(&cpu, &mut state, "stale", "mutex_permit_lock", &["pending"]).unwrap_err();
    assert!(stale.contains("revoked by shared close"));

    shared_mutex(&cpu, &mut state, "leased");
    let permit = execute(
        &cpu,
        &mut state,
        "active_permit",
        "mutex_permit",
        &["leased_shared", "lane1"],
    )
    .expect("issue active permit");
    state.bind_value("active_permit", permit);
    let lease = execute(
        &cpu,
        &mut state,
        "lease",
        "mutex_permit_lock",
        &["active_permit"],
    )
    .expect("lock permit");
    state.bind_value("lease", lease);
    let contended_permit = execute(
        &cpu,
        &mut state,
        "contended_permit",
        "mutex_permit",
        &["leased_shared", "lane0"],
    )
    .expect("issue permit while another lease is active");
    state.bind_value("contended_permit", contended_permit);
    let contention = execute(
        &cpu,
        &mut state,
        "contended_lease",
        "mutex_permit_lock",
        &["contended_permit"],
    )
    .unwrap_err();
    assert!(contention.contains("already has an active lease"));
    let active = execute(
        &cpu,
        &mut state,
        "early_close",
        "mutex_shared_close",
        &["leased_shared"],
    )
    .unwrap_err();
    assert!(active.contains("while a lease is active"));
    execute(
        &cpu,
        &mut state,
        "release",
        "mutex_lease_unlock",
        &["lease"],
    )
    .expect("release lease");
    let resumed_lease = execute(
        &cpu,
        &mut state,
        "resumed_lease",
        "mutex_permit_lock",
        &["contended_permit"],
    )
    .expect("contention must not consume the permit");
    state.bind_value("resumed_lease", resumed_lease);
    execute(
        &cpu,
        &mut state,
        "resumed_release",
        "mutex_lease_unlock",
        &["resumed_lease"],
    )
    .expect("release resumed lease");
    assert_eq!(
        execute(
            &cpu,
            &mut state,
            "final_close",
            "mutex_shared_close",
            &["leased_shared"],
        )
        .expect("close released mutex"),
        Value::Int(0)
    );
}
