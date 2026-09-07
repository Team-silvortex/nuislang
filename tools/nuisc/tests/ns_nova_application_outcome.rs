use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};
use yir_core::{ApplicationOutcome, Value, YirModule};
use yir_runtime_host::ApplicationSession;

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let project = Self(std::env::temp_dir().join(format!(
            "ns-outcome-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&project.0).unwrap();
        fs::write(
            project.0.join("nuis.toml"),
            concat!(
            "name = \"ns_nova_outcome\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\n",
            "galaxy = [\"ns-nova=workspace\"]\n",
            "galaxy_imports = [\"ns-nova:lib/app_runtime.ns\"]\n",
            "application_sessions = [\"report open=observe event=keep close=finish state=state\", ",
            "\"parent open=start_parent event=report_child close=finish_parent state=state\"]\n",
        ),
        )
        .unwrap();
        fs::write(project.0.join("main.ns"), SOURCE).unwrap();
        project
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const SOURCE: &str = r#"
use cpu NovaAppRuntime;
mod cpu Main {
  struct OutcomeView {
    status: i64, cleanup: i64, failure: i64, succeeded: i64, failed: i64
  }
  fn observe(status: i64, cleanup: i64, failure: i64) -> OutcomeView {
    let outcome: NovaAppOutcome = NovaAppRuntime.terminal_outcome(status, cleanup, failure);
    return OutcomeView {
      status: outcome.status,
      cleanup: outcome.cleanup_completed,
      failure: outcome.failure_kind,
      succeeded: if NovaAppRuntime.outcome_succeeded(outcome) { 1 } else { 0 },
      failed: if NovaAppRuntime.outcome_failed(outcome) { 1 } else { 0 }
    };
  }
  fn keep(state: OutcomeView) -> OutcomeView { return state; }
  fn finish(state: OutcomeView) -> OutcomeView { return state; }
  struct ParentView { seen: i64, status: i64, cleanup: i64, failure: i64 }
  fn start_parent(seed: i64) -> ParentView {
    return ParentView { seen: seed, status: 0, cleanup: 0, failure: 0 };
  }
  fn report_child(state: ParentView, status: i64, cleanup: i64, failure: i64) -> ParentView {
    let outcome: NovaAppOutcome = NovaAppRuntime.terminal_outcome(status, cleanup, failure);
    return ParentView {
      seen: state.seen + 1, status: outcome.status,
      cleanup: outcome.cleanup_completed, failure: outcome.failure_kind
    };
  }
  fn finish_parent(state: ParentView) -> ParentView { return state; }
  fn main() -> i64 {
    let success: OutcomeView = observe(1, 1, 0);
    let late: OutcomeView = observe(2, 1, 11);
    let cleanup_failed: OutcomeView = observe(2, 0, 1);
    let contradiction: OutcomeView = observe(1, 1, 11);
    let unknown: OutcomeView = observe(2, 1, 255);
    let forged: NovaAppOutcome = NovaAppOutcome { status: 1, cleanup_completed: 0, failure_kind: 0 };
    let parent: ParentView = report_child(start_parent(42), 2, 1, 11);
    if success.succeeded == 1 && success.failed == 0 &&
      late.failed == 1 && late.succeeded == 0 && late.cleanup == 1 && late.failure == 11 &&
      cleanup_failed.failed == 1 && cleanup_failed.cleanup == 0 &&
      contradiction.status == 3 && contradiction.succeeded == 0 &&
      unknown.status == 3 && unknown.failed == 0 &&
      NovaAppRuntime.outcome_succeeded(forged) == false &&
      parent.seen == 43 && parent.status == 2 && parent.cleanup == 1 && parent.failure == 11 {
      return 0;
    }
    return 1;
  }
}
"#;

fn consumer() -> &'static YirModule {
    static MODULE: OnceLock<YirModule> = OnceLock::new();
    MODULE.get_or_init(|| {
        nuisc::pipeline::compile_project(&Project::new().0)
            .unwrap()
            .yir
    })
}

fn field(value: &Value, name: &str) -> i64 {
    let Value::Struct(value) = value else {
        panic!("expected aggregate")
    };
    let (_, Value::Int(value)) = value.fields.iter().find(|(key, _)| key == name).unwrap() else {
        panic!("expected scalar field {name}")
    };
    *value
}

fn consume(codes: [i64; 3]) -> [i64; 5] {
    let registry = yir_verify::default_registry();
    // This is a separate test consumer, NOT a callback on the closed application.
    let (mut consumer, trace) = ApplicationSession::open_registered(
        consumer(),
        &registry,
        "report",
        codes.into_iter().map(Value::Int).collect(),
    )
    .unwrap();
    assert!(trace.presented_frames.is_empty());
    assert!(trace.provider_completion_witnesses.is_empty());
    let result = ["status", "cleanup", "failure", "succeeded", "failed"]
        .map(|name| field(consumer.state(), name));
    consumer.close(vec![]).unwrap();
    consumer.completion_status().unwrap();
    result
}

#[test]
fn compiled_nuis_decoder_matches_shared_terminal_contract() {
    let mut cases = vec![[1, 1, 0]];
    for failure in 1..=11 {
        cases.extend([[2, 0, failure], [2, 1, failure], [1, 1, failure]]);
    }
    cases.extend([
        [0, 0, 0],
        [1, 0, 0],
        [2, 1, 0],
        [2, 2, 11],
        [2, -1, 11],
        [3, 1, 11],
        [i64::MAX, 1, 11],
        [2, 1, -1],
        [2, 1, 12],
        [2, 1, 255],
        [2, 1, i64::MIN],
        [2, 1, i64::MAX],
    ]);
    for [status, cleanup, failure] in cases {
        let expected = match ApplicationOutcome::from_codes(status, cleanup, failure) {
            Ok(outcome) => [
                status,
                cleanup,
                failure,
                i64::from(outcome.is_success()),
                i64::from(!outcome.is_success()),
            ],
            Err(_) => [3, 0, 255, 0, 0],
        };
        assert_eq!(
            consume([status, cleanup, failure]),
            expected,
            "{status}/{cleanup}/{failure}"
        );
    }
}

#[test]
fn terminal_outcome_validation_runs_as_native_nuis_binary() {
    let project = Project::new();
    let output = project.0.join("out");
    let compiled = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .arg("compile")
        .arg(&project.0)
        .arg(&output)
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert_eq!(
        Command::new(output.join("ns_nova_outcome"))
            .status()
            .unwrap()
            .code(),
        Some(0)
    );
}

#[test]
fn compiled_image_parent_uses_rooted_initialization_and_owns_its_outcome_state() {
    let module = nuisc::pipeline::compile_project(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/projects/domains/ns_nova_image_showcase"),
    )
    .unwrap()
    .yir;
    let registry = yir_verify::default_registry();
    yir_runtime_host::validate_outcome_parent(&module, "parent").unwrap();
    let (mut parent, opened) = ApplicationSession::open_registered_rooted_budgeted(
        &module,
        &registry,
        "parent",
        vec![],
        100_000,
    )
    .unwrap();
    assert_eq!(field(parent.state(), "outcomes"), 0);
    assert!(opened.presented_frames.is_empty());
    assert!(opened.provider_completion_witnesses.is_empty());
    // No child provider domain may be initialized in the CPU parent's context.
    assert!(opened
        .lane_steps
        .values()
        .flatten()
        .all(|step| step.starts_with("cpu.")));
    parent
        .event_budgeted(vec![Value::Int(2), Value::Int(1), Value::Int(11)], 100_000)
        .unwrap();
    assert_eq!(field(parent.state(), "outcomes"), 1);
    assert_eq!(field(parent.state(), "status"), 2);
    assert_eq!(field(parent.state(), "cleanup"), 1);
    assert_eq!(field(parent.state(), "failure"), 11);
    parent.close_budgeted(vec![], 100_000).unwrap();
    parent.completion_status().unwrap();
}

#[cfg(unix)]
#[test]
fn actual_late_finish_snapshot_reaches_nuis_without_reopening_the_image_session() {
    use std::{
        os::unix::net::UnixListener,
        thread,
        time::{Duration, Instant},
    };
    use yir_core::provider_runtime_ipc::{
        hash_bytes, DispatchTarget, Message, Rejection, RejectionCode, RejectionPhase,
    };
    use yir_runtime_host::{ApplicationProviderSource, ApplicationPumpPhase, WindowSession};

    // This parent is already running before the child starts. Delivery invokes
    // its ordinary Nuis event, not a new context or a post-close child callback.
    let registry = yir_verify::default_registry();
    let (mut parent, _) =
        ApplicationSession::open_registered(consumer(), &registry, "parent", vec![Value::Int(42)])
            .unwrap();
    parent
        .event(vec![Value::Int(1), Value::Int(1), Value::Int(0)])
        .unwrap();
    assert_eq!(field(parent.state(), "seen"), 43);
    let module = nuisc::pipeline::compile_project(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/projects/domains/ns_nova_image_showcase"),
    )
    .unwrap()
    .yir;
    let source = nuisc::render::render_yir(&module);
    let node = module
        .nodes
        .iter()
        .find(|node| node.op.module == "shader" && node.op.instruction == "draw_instanced")
        .unwrap();
    let target = DispatchTarget {
        source_yir_fnv1a64: hash_bytes(source.as_bytes()),
        module: node.op.module.clone(),
        instruction: node.op.instruction.clone(),
        node: node.name.clone(),
        resource: node.resource.clone(),
    };
    let project = Project::new();
    let socket = project.0.join("peer");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let worker = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "consumer never connected");
                    thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("{error}"),
            }
        };
        // BSD accept can inherit the listener's nonblocking mode; per-stream
        // deadlines only apply after restoring blocking I/O on the accepted peer.
        stream.set_nonblocking(false).unwrap();
        stream
            // Includes two independent debug-executor callbacks before Finish;
            // unlike production idle waiting, this fixture has a total read cap.
            .set_read_timeout(Some(Duration::from_secs(60)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(60)))
            .unwrap();
        Message::Hello(target).write_to(&mut stream).unwrap();
        // Opening/closing the image state must not dispatch or replay a frame.
        assert!(matches!(
            Message::read_from(&mut stream).unwrap(),
            Message::Finish(0)
        ));
        Message::Rejected(Rejection {
            phase: RejectionPhase::Finish,
            sequence: 0,
            code: RejectionCode::Finalization,
            detail: "publication failed".to_owned(),
        })
        .write_to(&mut stream)
        .unwrap();
        assert!(
            Message::read_from(&mut stream).is_err(),
            "unexpected work after Finish rejection"
        );
    });
    let mut session = WindowSession::spawn(
        source,
        ApplicationProviderSource::Ipc(&socket),
        "window".to_owned(),
        640,
        400,
    )
    .unwrap();
    let receive = |session: &mut WindowSession| {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(reply) = session.poll().unwrap() {
                break reply;
            }
            assert!(Instant::now() < deadline, "window reply timed out");
            thread::sleep(Duration::from_millis(1));
        }
    };
    assert!(receive(&mut session).frame.unwrap().is_none());
    assert!(session.take_outcome_delivery().is_none());
    session.close().unwrap();
    assert_eq!(session.outcome(), None);
    let closed = receive(&mut session);
    assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
    let detail = closed.frame.unwrap_err();
    assert!(detail.contains("publication failed"), "{detail}");
    let old_state = closed.state.unwrap();
    assert_eq!(field(&old_state, "status"), 2);
    assert_eq!(field(&old_state, "failure_kind"), 0);
    let outcome = closed.outcome.unwrap();
    assert_eq!(outcome.codes(), [2, 1, 11]);
    assert_eq!(consume(outcome.codes()), [2, 1, 11, 0, 1]);
    let delivery = session.take_outcome_delivery().unwrap();
    assert_eq!(delivery.outcome(), outcome);
    assert!(session.take_outcome_delivery().is_none());
    let delivered = delivery.deliver(&mut parent, 100_000).unwrap();
    assert!(delivered.presented_frames.is_empty());
    assert!(delivered.provider_completion_witnesses.is_empty());
    assert_eq!(field(parent.state(), "seen"), 44);
    assert_eq!(field(parent.state(), "status"), 2);
    assert_eq!(field(parent.state(), "cleanup"), 1);
    assert_eq!(field(parent.state(), "failure"), 11);
    // A failed child report does not fault the independent parent lifecycle.
    assert_eq!(
        parent.failure_kind(),
        yir_core::ApplicationFailureKind::None
    );
    parent.close(vec![]).unwrap();
    parent.completion_status().unwrap();
    assert_eq!(session.outcome(), Some(outcome));
    assert!(session.close().is_err());
    assert!(session.event(0, 0).is_err());
    assert!(session.poll().unwrap().is_none());
    assert!(session.take_outcome_delivery().is_none());
    drop(session);
    worker.join().unwrap();
}
