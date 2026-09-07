use std::{collections::BTreeSet, path::PathBuf, sync::OnceLock};

use yir_core::{Value, YirFunctionRole, YirModule};
use yir_runtime_host::{ApplicationSession, ApplicationSessionPhase};

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
    let (mut session, opened) = ApplicationSession::open_registered(
        module,
        &registry,
        "image",
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

#[test]
fn compiled_registration_roundtrips_and_rejects_signature_drift() {
    let module = image_module();
    let source = nuisc::render::render_yir(module);
    let roundtrip = yir_syntax::parse_module(&source).unwrap();
    assert_eq!(roundtrip.application_sessions, module.application_sessions);
    assert_eq!(roundtrip.application_sessions[0].id, "image");
    yir_verify::verify_module(&roundtrip).unwrap();
    let mut drift = roundtrip.clone();
    drift.application_sessions[0].event = "missing_event".to_owned();
    assert!(yir_verify::verify_module(&drift)
        .unwrap_err()
        .contains("unknown function"));
    let mut duplicate = roundtrip;
    duplicate
        .application_sessions
        .push(duplicate.application_sessions[0].clone());
    assert!(yir_verify::verify_module(&duplicate)
        .unwrap_err()
        .contains("duplicate application session"));
}

#[test]
fn compiled_nuis_close_reason_sets_and_preserves_failure_without_frame_replay() {
    let module = image_module();
    let registry = yir_verify::default_registry();
    let legacy_close = &yir_core::registered_application_session(module, "image")
        .unwrap()
        .close;
    let window = yir_core::registered_application_session(module, "window").unwrap();
    for reason in [0, 1, 2, -1, 99] {
        // Reference-only helper composition checks std failure-state semantics;
        // the WindowSession transport tests separately enforce failure admission.
        let (mut session, _) = ApplicationSession::open(
            module,
            &registry,
            yir_runtime_host::ApplicationSessionEntries {
                open: &window.open,
                event: &window.close,
                close: legacy_close,
                state_parameter: "state",
            },
            vec![Value::Int(640), Value::Int(400)],
        )
        .unwrap();
        let trace = session.event(vec![Value::Int(reason)]).unwrap();
        let status = if reason == 0 { 2 } else { 4 };
        assert_eq!(field(session.state(), "status"), status);
        assert_eq!(field(session.state(), "frame_index"), 0);
        assert_eq!(field(session.state(), "last_completion_root"), 0);
        assert!(trace.presented_frames.is_empty());
        assert!(trace.provider_completion_witnesses.is_empty());
        // A subsequent ordinary std cleanup must not clear an application failure.
        let trace = session.close(vec![Value::Int(10)]).unwrap().unwrap();
        assert_eq!(field(session.state(), "status"), status);
        assert!(trace.presented_frames.is_empty());
    }
}

#[test]
fn registration_preserves_uncalled_helpers_as_host_roots() {
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Project(PathBuf);
    impl Drop for Project {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let project = Project(std::env::temp_dir().join(format!(
        "nuis-registered-roots-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&project.0).unwrap();
    let manifest = "name = \"registered_roots\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\napplication_sessions = [\"counter open=start event=step close=stop state=state\"]\n";
    fs::write(project.0.join("nuis.toml"), manifest).unwrap();
    fs::write(
        project.0.join("main.ns"),
        r#"
mod cpu Main {
  struct Counter { count: i64 }
  fn start(seed: i64) -> Counter { return Counter { count: seed }; }
  fn step(state: Counter, input: i64) -> Counter {
    return Counter { count: state.count + input };
  }
  fn stop(state: Counter) -> Counter { return state; }
  fn main() { print(999); }
}
"#,
    )
    .unwrap();
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    let (mut session, opened) = ApplicationSession::open_registered(
        &compiled.yir,
        &registry,
        "counter",
        vec![Value::Int(40)],
    )
    .unwrap();
    let entry_nodes = compiled
        .yir
        .functions
        .iter()
        .filter(|function| function.role == YirFunctionRole::Entry)
        .flat_map(|function| function.body_nodes.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    assert!(!entry_nodes.is_empty());
    for step in opened.lane_steps.values().flatten() {
        assert!(
            !entry_nodes.contains(step.rsplit_once(" -> ").unwrap().1),
            "registration replayed main: {step}"
        );
    }
    session.event(vec![Value::Int(2)]).unwrap();
    assert_eq!(field(session.state(), "count"), 42);
    session.close(vec![]).unwrap();
    session.completion_status().unwrap();

    fs::write(
        project.0.join("nuis.toml"),
        manifest.replace("event=step", "event=missing"),
    )
    .unwrap();
    let error = nuisc::pipeline::compile_project(&project.0).err().unwrap();
    assert!(error.contains("host entry function `missing`"), "{error}");
}
