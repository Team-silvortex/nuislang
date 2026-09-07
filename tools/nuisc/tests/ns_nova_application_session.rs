use std::{collections::BTreeSet, path::PathBuf, sync::OnceLock};

use yir_core::{Value, YirFunctionRole, YirModule};
use yir_runtime_host::{ApplicationSession, ApplicationSessionEntries, ApplicationSessionPhase};

fn image_module() -> &'static YirModule {
    static MODULE: OnceLock<YirModule> = OnceLock::new();
    MODULE.get_or_init(|| {
        let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/projects/domains/ns_nova_image_showcase");
        nuisc::pipeline::compile_project(&project).unwrap().yir
    })
}

fn field(state: &Value, name: &str) -> i64 {
    let Value::Struct(state) = state else {
        panic!("expected Nuis state")
    };
    let (_, Value::Int(value)) = state
        .fields
        .iter()
        .find(|(field, _)| field == name)
        .unwrap()
    else {
        panic!("expected scalar state field {name}")
    };
    *value
}

#[test]
fn compiled_nuis_image_state_survives_independent_events_without_main_replay() {
    let module = image_module();
    let registry = yir_verify::default_registry();
    let entries = ApplicationSessionEntries {
        open: "NovaAppRuntime.open",
        event: "render_showcase_frame",
        close: "NovaAppRuntime.close",
        state_parameter: "state",
    };
    let (mut session, opened) = ApplicationSession::open(
        module,
        &registry,
        entries,
        vec![
            Value::Int(640),
            Value::Int(400),
            Value::Int(60),
            Value::Int(0),
        ],
    )
    .unwrap();
    assert_eq!(field(session.state(), "frame_index"), 0);
    assert!(opened.presented_frames.is_empty());

    let entry_nodes = module
        .functions
        .iter()
        .filter(|function| function.role == YirFunctionRole::Entry)
        .flat_map(|function| function.body_nodes.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let mut prior_clock = 0;
    let mut prior_root = 0;
    let mut traces = vec![opened];
    // Two separate host deliveries, deliberately skipping the middle image input.
    // These are reference-provider checks, not a claim of live Metal execution.
    for (ordinal, input) in [(1, 0), (2, 2)] {
        let trace = session.event(vec![Value::Int(input)]).unwrap();
        assert_eq!(field(session.state(), "frame_index"), ordinal);
        assert_eq!(field(session.state(), "presented_frames"), ordinal);
        assert_eq!(field(session.state(), "dropped_frames"), 0);
        assert_eq!(
            trace.presented_frames.len(),
            1,
            "frame history must be drained"
        );
        assert_eq!(trace.presented_frames[0].width, 160);
        assert_eq!(trace.presented_frames[0].height, 120);
        let clock = field(session.state(), "last_completion_tick");
        let root = field(session.state(), "last_completion_root");
        assert!(
            clock > prior_clock,
            "provider clock must survive independent calls"
        );
        assert!(root > 0);
        if prior_root != 0 {
            assert_eq!(root, prior_root);
        }
        prior_clock = clock;
        prior_root = root;
        traces.push(trace);
    }
    let closed = session
        .close(vec![Value::Int(prior_clock + 1)])
        .unwrap()
        .unwrap();
    assert_eq!(session.phase(), ApplicationSessionPhase::Closed);
    assert_eq!(field(session.state(), "status"), 2);
    assert_eq!(field(session.state(), "frame_index"), 2);
    assert!(closed.presented_frames.is_empty());
    assert!(session.close(vec![Value::Int(999)]).unwrap().is_none());
    assert!(session.event(vec![Value::Int(1)]).is_err());
    traces.push(closed);
    for trace in traces {
        assert!(
            trace.values.is_empty(),
            "session traces must not export heap capabilities"
        );
        for step in trace.lane_steps.values().flatten() {
            let node = step.rsplit_once(" -> ").unwrap().1;
            assert!(!entry_nodes.contains(node), "replayed main node {node}");
        }
    }
}
