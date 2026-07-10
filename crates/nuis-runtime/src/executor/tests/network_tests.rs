use nuis_artifact::{ClockDomain, ClockEdge};

use crate::ExecutionResourceBinding;

use super::super::{
    domain_resource_capability_label, ExecutionContract, ExecutionProfile, ExecutionResourceKind,
    Executor,
};
use super::support::*;

#[test]
fn verify_rejects_incomplete_contract() {
    let contract = ExecutionContract {
        yir_version: "",
        fabric_abi_version: "federated-abi-v1",
        profile: ExecutionProfile::Aot,
    };

    assert_eq!(
        Executor.verify(&contract),
        Err("execution contract is incomplete")
    );
}

#[test]
fn executor_plans_phase_bindings_from_prepared_execution() {
    let adapter = NetworkAdapter;
    let unit = sample_network_unit();
    let payload = sample_network_payload();
    let host_plan = sample_network_host_plan();
    let bridge_registry = sample_network_bridge_registry();
    let prepared =
        prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

    let plan = Executor.plan(&prepared).unwrap();

    assert_eq!(plan.domain_family, "network");
    assert_eq!(plan.package_id, "official.network");
    assert_eq!(plan.adapter_id, "network-test-adapter");
    assert_eq!(
        plan.selected_lowering_target.as_deref(),
        Some("urlsession.socket-io")
    );
    assert_eq!(plan.phases.len(), 4);
    assert_eq!(plan.phases[0].phase, "bind");
    assert_eq!(plan.phases[0].role, crate::RuntimeRole::Bind);
    assert_eq!(plan.phases[0].bridge_surface, "host-ffi.bridge.network");
    assert_eq!(plan.phases[0].scheduler_binding, "network-poll-bridge");
    assert_eq!(
        plan.phases[0].lowering_summary,
        "execution_route = \"foundation-session-reactor\""
    );
    assert_eq!(
        plan.phases[0].backend_summary,
        "transport_ir = \"foundation-url-request\""
    );
    assert_eq!(
        plan.phases[0].bridge_summary,
        "phase_submit = \"packet-write-dispatch\""
    );
    assert_eq!(
        plan.phases[0].ir_sidecar_summary.as_deref(),
        Some("schema = \"nuis-network-ir-sidecar-v1\"")
    );
    assert_eq!(plan.phases[0].action.kind, "network.bind");
    assert_eq!(
        plan.phases[0].action.input_handles,
        vec!["authority.text".to_owned()]
    );
    assert_eq!(
        plan.phases[0].action.output_handles,
        vec!["session.handle".to_owned()]
    );
    assert_eq!(
        plan.phases[0].action.resolved_inputs,
        Vec::<ExecutionResourceBinding>::new()
    );
    assert_eq!(
        plan.phases[0].action.resolved_resources,
        Vec::<ExecutionResourceBinding>::new()
    );
    assert_eq!(
        plan.phases[0].action.adapter_hint.as_deref(),
        Some("adapter.bind.session-open")
    );
    assert_eq!(plan.phases[2].action.kind, "network.wait");
    assert_eq!(
        plan.phases[2].action.adapter_hint.as_deref(),
        Some("adapter.wait.callback-poll")
    );

    let summary = plan.render_summary();
    assert!(summary.contains("selected_lowering_target = urlsession.socket-io"));
    assert!(summary.contains("phase bind role=Bind"));
    assert!(summary.contains(
        "resource bridge_surface kind=Bridge capability=cap.network.urlsession_socket_io.bridge.bridge_surface"
    ));
    assert!(summary.contains(
        "resource active_session kind=Handle capability=cap.network.urlsession_socket_io.handle.active_session"
    ));
}

#[test]
fn executor_exposes_clock_protocol_as_phase_metadata_resource() {
    let adapter = PassiveAdapter;
    let unit = sample_network_unit();
    let payload = sample_network_payload();
    let host_plan = sample_network_host_plan();
    let bridge_registry = sample_network_bridge_registry();
    let clock_domain = ClockDomain {
        index: 0,
        domain_family: "network".to_owned(),
        package_id: "official.network".to_owned(),
        clock_domain_id: "network.clock.io.v1".to_owned(),
        clock_kind: "io-monotonic".to_owned(),
        clock_epoch_kind: "io-epoch".to_owned(),
        clock_resolution: "io-ready-step".to_owned(),
        clock_bridge_default: "global->io:bridge".to_owned(),
        lifecycle_hook: "on_network_bridge_progress".to_owned(),
    };
    let clock_edge = ClockEdge {
        index: 1,
        from: "t0000.nuis.bootstrap.lifecycle.v1".to_owned(),
        to: "t0001.network".to_owned(),
        relation: "happens-before".to_owned(),
        source: "hetero.node.0".to_owned(),
    };
    let data_commit_edge = ClockEdge {
        index: 2,
        from: "t0001.network.complete".to_owned(),
        to: "t0001.network.data_commit".to_owned(),
        relation: "data-segment-commit".to_owned(),
        source: "hetero.data_segment.0".to_owned(),
    };
    let mut prepared =
        prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);
    prepared.clock_domain = Some(&clock_domain);
    prepared.clock_edges = vec![&clock_edge, &data_commit_edge];

    let plan = Executor.plan(&prepared).unwrap();

    assert_eq!(
        plan.clock_gate.wait_on,
        vec!["t0000.nuis.bootstrap.lifecycle.v1".to_owned()]
    );
    let expected_emits = [
        "t0001.network",
        "t0001.network.complete",
        "t0001.network.data_commit",
    ]
    .map(str::to_owned)
    .to_vec();
    assert_eq!(plan.clock_gate.emits, expected_emits);
    let trace = Executor.execute_prepared_plan(&adapter, &plan).unwrap();
    assert_eq!(trace.events[0].clock_gate, plan.clock_gate);
    let validation = trace
        .validate_clock_gates(&["t0000.nuis.bootstrap.lifecycle.v1".to_owned()])
        .unwrap();
    assert_eq!(
        validation.initial_timestamps,
        vec!["t0000.nuis.bootstrap.lifecycle.v1".to_owned()]
    );
    assert_eq!(validation.observed_emits, expected_emits);
    assert!(validation
        .final_timestamps
        .contains(&"t0001.network.data_commit".to_owned()));
    let error = trace.validate_clock_gates(&[]).unwrap_err();
    assert!(error
        .to_string()
        .contains("missing timestamp `t0000.nuis.bootstrap.lifecycle.v1`"));
}

#[test]
fn executor_emits_trace_for_prepared_execution() {
    let adapter = NetworkAdapter;
    let unit = sample_network_unit();
    let payload = sample_network_payload();
    let host_plan = sample_network_host_plan();
    let bridge_registry = sample_network_bridge_registry();
    let prepared =
        prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

    let trace = Executor.execute_prepared(&prepared).unwrap();

    assert_eq!(trace.domain_family, "network");
    assert_eq!(trace.phase_count, 4);
    assert_eq!(trace.events[0].phase, "bind");
    assert_eq!(trace.events[0].role, crate::RuntimeRole::Bind);
    assert_eq!(trace.events[1].phase, "submit");
    assert_eq!(trace.events[1].role, crate::RuntimeRole::Execute);
    assert_eq!(trace.events[1].action.kind, "network.submit");
    assert_eq!(
        trace.events[1].action.adapter_hint.as_deref(),
        Some("adapter.submit.request-dispatch")
    );
    assert_eq!(
        trace.events[0].state_before.available_handles,
        Vec::<String>::new()
    );
    assert_eq!(
        trace.events[0].state_after.available_handles,
        vec!["session.handle".to_owned()]
    );
    assert_eq!(
        trace.events[0].state_after.handle_slots,
        vec![ExecutionResourceBinding {
            key: "session.handle".to_owned(),
            kind: ExecutionResourceKind::Handle,
            capability_label: Some("cap.handle.session.handle".to_owned()),
            value: "network://bind/session.handle".to_owned()
        }]
    );
    assert_eq!(
        trace.events[1].state_before.available_handles,
        vec!["session.handle".to_owned()]
    );
    assert_eq!(
        trace.events[1].state_before.handle_slots,
        vec![ExecutionResourceBinding {
            key: "session.handle".to_owned(),
            kind: ExecutionResourceKind::Handle,
            capability_label: Some("cap.handle.session.handle".to_owned()),
            value: "network://bind/session.handle".to_owned()
        }]
    );
    assert_eq!(
        trace.events[1].action.input_handles,
        vec!["session.handle".to_owned(), "request.packet".to_owned()]
    );
    assert_eq!(
        trace.events[1].action.resolved_inputs,
        vec![ExecutionResourceBinding {
            key: "session.handle".to_owned(),
            kind: ExecutionResourceKind::Handle,
            capability_label: Some("cap.handle.session.handle".to_owned()),
            value: "network://bind/session.handle".to_owned()
        }]
    );
    assert_eq!(
        trace.events[1].action.resolved_resources,
        vec![
            ExecutionResourceBinding {
                key: "bridge_surface".to_owned(),
                kind: ExecutionResourceKind::Bridge,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "bridge_surface",
                    &ExecutionResourceKind::Bridge,
                )),
                value: "host-ffi.bridge.network".to_owned()
            },
            ExecutionResourceBinding {
                key: "scheduler_binding".to_owned(),
                kind: ExecutionResourceKind::Scheduler,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "scheduler_binding",
                    &ExecutionResourceKind::Scheduler,
                )),
                value: "network-poll-bridge".to_owned()
            },
            ExecutionResourceBinding {
                key: "backend_summary".to_owned(),
                kind: ExecutionResourceKind::Metadata,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "backend_summary",
                    &ExecutionResourceKind::Metadata,
                )),
                value: "transport_ir = \"foundation-url-request\"".to_owned()
            },
            ExecutionResourceBinding {
                key: "active_session".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "active_session",
                    &ExecutionResourceKind::Handle,
                )),
                value: "network://bind/session.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "active_task".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "active_task",
                    &ExecutionResourceKind::Handle,
                )),
                value: "unresolved:task.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "active_response".to_owned(),
                kind: ExecutionResourceKind::Response,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "active_response",
                    &ExecutionResourceKind::Response,
                )),
                value: "unresolved:response.handle".to_owned()
            }
        ]
    );
    assert_eq!(trace.events[1].outcome.status, "adapter-submit");
    assert_eq!(
        trace.events[1].outcome.produced_handles,
        vec!["task.handle".to_owned()]
    );
    assert_eq!(
        trace.events[1].outcome.produced_slots,
        vec![ExecutionResourceBinding {
            key: "task.handle".to_owned(),
            kind: ExecutionResourceKind::Handle,
            capability_label: Some("cap.handle.task.handle".to_owned()),
            value: "network://submit/task.handle".to_owned()
        }]
    );
    assert_eq!(
        trace.events[1].state_after.available_handles,
        vec!["session.handle".to_owned(), "task.handle".to_owned()]
    );
    assert_eq!(
        trace.events[1].state_after.handle_slots,
        vec![
            ExecutionResourceBinding {
                key: "session.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.session.handle".to_owned()),
                value: "network://bind/session.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "task.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.task.handle".to_owned()),
                value: "network://submit/task.handle".to_owned()
            }
        ]
    );
    assert_eq!(trace.events[3].phase, "finalize");
    assert_eq!(trace.events[3].bridge_surface, "host-ffi.bridge.network");
    assert_eq!(trace.events[3].scheduler_binding, "network-poll-bridge");
    assert_eq!(trace.events[3].adapter_id, "network-test-adapter");
    assert_eq!(
        trace.events[3].action.adapter_hint.as_deref(),
        Some("adapter.finalize.response-commit")
    );
    assert_eq!(trace.events[3].outcome.status, "adapter-finalize");
    assert_eq!(
        trace.events[3].outcome.notes,
        vec![
            "domain=network".to_owned(),
            "kind=network.finalize".to_owned()
        ]
    );
    assert_eq!(
        trace.events[3].state_before.available_handles,
        vec![
            "session.handle".to_owned(),
            "task.handle".to_owned(),
            "response.handle".to_owned()
        ]
    );
    assert_eq!(
        trace.events[3].action.resolved_inputs,
        vec![ExecutionResourceBinding {
            key: "response.handle".to_owned(),
            kind: ExecutionResourceKind::Handle,
            capability_label: Some("cap.handle.response.handle".to_owned()),
            value: "network://wait/response.handle".to_owned()
        }]
    );
    assert_eq!(
        trace.events[3].action.resolved_resources,
        vec![
            ExecutionResourceBinding {
                key: "bridge_surface".to_owned(),
                kind: ExecutionResourceKind::Bridge,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "bridge_surface",
                    &ExecutionResourceKind::Bridge,
                )),
                value: "host-ffi.bridge.network".to_owned()
            },
            ExecutionResourceBinding {
                key: "scheduler_binding".to_owned(),
                kind: ExecutionResourceKind::Scheduler,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "scheduler_binding",
                    &ExecutionResourceKind::Scheduler,
                )),
                value: "network-poll-bridge".to_owned()
            },
            ExecutionResourceBinding {
                key: "backend_summary".to_owned(),
                kind: ExecutionResourceKind::Metadata,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "backend_summary",
                    &ExecutionResourceKind::Metadata,
                )),
                value: "transport_ir = \"foundation-url-request\"".to_owned()
            },
            ExecutionResourceBinding {
                key: "active_session".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "active_session",
                    &ExecutionResourceKind::Handle,
                )),
                value: "network://bind/session.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "active_task".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "active_task",
                    &ExecutionResourceKind::Handle,
                )),
                value: "network://submit/task.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "active_response".to_owned(),
                kind: ExecutionResourceKind::Response,
                capability_label: Some(domain_resource_capability_label(
                    "network",
                    Some("urlsession.socket-io"),
                    "active_response",
                    &ExecutionResourceKind::Response,
                )),
                value: "network://wait/response.handle".to_owned()
            }
        ]
    );
    assert_eq!(
        trace.events[3].state_after.available_handles,
        vec![
            "session.handle".to_owned(),
            "task.handle".to_owned(),
            "response.handle".to_owned(),
            "status.code".to_owned()
        ]
    );
    assert_eq!(
        trace.events[3].state_after.handle_slots,
        vec![
            ExecutionResourceBinding {
                key: "response.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.response.handle".to_owned()),
                value: "network://wait/response.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "session.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.session.handle".to_owned()),
                value: "network://bind/session.handle".to_owned()
            },
            ExecutionResourceBinding {
                key: "status.code".to_owned(),
                kind: ExecutionResourceKind::Slot,
                capability_label: Some("cap.slot.status.code".to_owned()),
                value: "network://finalize/status.code".to_owned()
            },
            ExecutionResourceBinding {
                key: "task.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.task.handle".to_owned()),
                value: "network://submit/task.handle".to_owned()
            }
        ]
    );

    let summary = trace.render_summary();
    assert!(summary.contains("event submit role=Execute adapter=network-test-adapter"));
    assert!(summary.contains(
        "resolved_resource active_session kind=Handle capability=cap.network.urlsession_socket_io.handle.active_session value=network://bind/session.handle"
    ));
    assert!(summary.contains(
        "produced_slot task.handle kind=Handle capability=cap.handle.task.handle value=network://submit/task.handle"
    ));
}
