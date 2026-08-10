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
    shared_mutex_with_cardinality(cpu, state, prefix, 2);
}

fn shared_mutex_with_cardinality(
    cpu: &CpuMod,
    state: &mut ExecutionState,
    prefix: &str,
    permit_cardinality: i64,
) {
    let value_name = format!("{prefix}_value");
    let mutex_name = format!("{prefix}_mutex");
    let shared_name = format!("{prefix}_shared");
    let cardinality_name = format!("{prefix}_cardinality");
    state.bind_value(&value_name, Value::Int(23));
    state.bind_value(&cardinality_name, Value::Int(permit_cardinality));
    let mutex =
        execute(cpu, state, &mutex_name, "mutex_new", &[&value_name]).expect("create mutex");
    state.bind_value(&mutex_name, mutex);
    let shared = execute(
        cpu,
        state,
        &shared_name,
        "mutex_share",
        &[&mutex_name, &cardinality_name],
    )
    .expect("share mutex");
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

#[test]
fn static_cardinality_admits_three_lanes_and_rejects_the_fourth() {
    let cpu = CpuMod;
    let mut state = ExecutionState::default();
    for lane in 0..=3 {
        state.bind_value(&format!("lane{lane}"), Value::Int(lane));
    }
    shared_mutex_with_cardinality(&cpu, &mut state, "wide", 3);

    for lane in 0..=2 {
        let name = format!("permit{lane}");
        let lane_name = format!("lane{lane}");
        let permit = execute(
            &cpu,
            &mut state,
            &name,
            "mutex_permit",
            &["wide_shared", &lane_name],
        )
        .expect("issue configured permit lane");
        state.bind_value(&name, permit);
    }
    let error = execute(
        &cpu,
        &mut state,
        "permit3",
        "mutex_permit",
        &["wide_shared", "lane3"],
    )
    .unwrap_err();
    assert!(error.contains("outside configured range `0..3`"));
    assert_eq!(
        state.shared_mutex_permit_cardinalities.get("wide_mutex"),
        Some(&3)
    );
    assert_eq!(
        execute(
            &cpu,
            &mut state,
            "wide_close",
            "mutex_shared_close",
            &["wide_shared"],
        )
        .expect("close three-lane mutex"),
        Value::Int(3)
    );
    assert!(!state
        .shared_mutex_permit_cardinalities
        .contains_key("wide_mutex"));
}

#[test]
fn lease_replace_publishes_value_across_preissued_permits() {
    let cpu = CpuMod;
    let mut state = ExecutionState::default();
    state.bind_value("lane0", Value::Int(0));
    state.bind_value("lane1", Value::Int(1));
    state.bind_value("replacement", Value::Int(31));
    shared_mutex(&cpu, &mut state, "replace");

    for (name, lane) in [("left_permit", "lane0"), ("right_permit", "lane1")] {
        let permit = execute(
            &cpu,
            &mut state,
            name,
            "mutex_permit",
            &["replace_shared", lane],
        )
        .expect("issue permit before mutation");
        state.bind_value(name, permit);
    }

    let left_lease = execute(
        &cpu,
        &mut state,
        "left_lease",
        "mutex_permit_lock",
        &["left_permit"],
    )
    .expect("lock first permit");
    state.bind_value("left_lease", left_lease);
    assert_eq!(
        execute(
            &cpu,
            &mut state,
            "old",
            "mutex_lease_replace",
            &["left_lease", "replacement"],
        )
        .expect("replace leased value"),
        Value::Int(23)
    );
    assert_eq!(
        execute(
            &cpu,
            &mut state,
            "left_value",
            "mutex_lease_value",
            &["left_lease"],
        )
        .expect("read replacement through first lease"),
        Value::Int(31)
    );
    execute(
        &cpu,
        &mut state,
        "left_release",
        "mutex_lease_unlock",
        &["left_lease"],
    )
    .expect("release first lease");

    let right_lease = execute(
        &cpu,
        &mut state,
        "right_lease",
        "mutex_permit_lock",
        &["right_permit"],
    )
    .expect("lock preissued second permit");
    state.bind_value("right_lease", right_lease);
    assert_eq!(
        execute(
            &cpu,
            &mut state,
            "right_value",
            "mutex_lease_value",
            &["right_lease"],
        )
        .expect("observe published replacement"),
        Value::Int(31)
    );
    execute(
        &cpu,
        &mut state,
        "right_release",
        "mutex_lease_unlock",
        &["right_lease"],
    )
    .expect("release second lease");

    assert_eq!(
        state.shared_mutex_values.get("replace_mutex"),
        Some(&Value::Int(31))
    );
    assert_eq!(
        state.shared_mutex_release_epochs.get("replace_mutex"),
        Some(&3)
    );
}

#[test]
fn lease_replace_preserves_i32_payload_kind() {
    let cpu = CpuMod;
    let mut state = ExecutionState::default();
    state.bind_value("value", Value::I32(-17));
    state.bind_value("replacement", Value::I32(23));
    state.bind_value("wrong_kind", Value::Int(23));
    state.bind_value("cardinality", Value::Int(1));
    state.bind_value("lane", Value::Int(0));

    let mutex =
        execute(&cpu, &mut state, "i32_mutex", "mutex_new", &["value"]).expect("create i32 mutex");
    state.bind_value("i32_mutex", mutex);
    let shared = execute(
        &cpu,
        &mut state,
        "i32_shared",
        "mutex_share",
        &["i32_mutex", "cardinality"],
    )
    .expect("share i32 mutex");
    state.bind_value("i32_shared", shared);
    let permit = execute(
        &cpu,
        &mut state,
        "i32_permit",
        "mutex_permit",
        &["i32_shared", "lane"],
    )
    .expect("issue i32 permit");
    state.bind_value("i32_permit", permit);
    let lease = execute(
        &cpu,
        &mut state,
        "i32_lease",
        "mutex_permit_lock",
        &["i32_permit"],
    )
    .expect("lock i32 permit");
    state.bind_value("i32_lease", lease);

    let mismatch = execute(
        &cpu,
        &mut state,
        "i32_mismatch",
        "mutex_lease_replace",
        &["i32_lease", "wrong_kind"],
    )
    .unwrap_err();
    assert!(mismatch.contains("must preserve the native i32/i64 scalar payload kind"));

    assert_eq!(
        execute(
            &cpu,
            &mut state,
            "i32_old",
            "mutex_lease_replace",
            &["i32_lease", "replacement"],
        )
        .expect("replace i32 payload"),
        Value::I32(-17)
    );
    assert_eq!(
        execute(
            &cpu,
            &mut state,
            "i32_current",
            "mutex_lease_value",
            &["i32_lease"],
        )
        .expect("read i32 payload"),
        Value::I32(23)
    );
}
