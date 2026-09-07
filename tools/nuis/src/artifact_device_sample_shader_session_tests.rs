use super::*;
use std::{
    fs::DirBuilder,
    io::ErrorKind,
    os::unix::{fs::DirBuilderExt, net::UnixListener},
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
use yir_core::{ProviderCompletionClockKind, Value, YirFunctionRole};
use yir_runtime_host::{
    with_registered_provider_application_session, ApplicationEventPump, ApplicationProviderSource,
    ApplicationPumpOperation, ApplicationPumpPhase, ApplicationPumpReply,
};

#[path = "artifact_device_sample_shader_window_tests.rs"]
mod window;

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn configuration() -> Vec<Value> {
    [640, 400, 60, 0].map(Value::Int).to_vec()
}

fn state_field(state: &Value, name: &str) -> i64 {
    let Value::Struct(state) = state else {
        panic!("missing Nuis state")
    };
    let (_, Value::Int(value)) = state
        .fields
        .iter()
        .find(|(field, _)| field == name)
        .unwrap()
    else {
        panic!("missing scalar {name}")
    };
    *value
}

#[test]
fn executes_ns_nova_persistent_image_session_through_live_provider() {
    let output = Artifacts(temp_output_dir());
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    crate::handle_build(
        workspace.join("examples/projects/domains/ns_nova_image_showcase"),
        output.0.clone(),
        false,
        None,
        None,
        None,
    )
    .expect("compile the Nuis image project");
    let prepared =
        crate::artifact_runtime_provider_results::prepare_runtime_provider_results(&output.0)
            .unwrap()
            .unwrap();
    assert!(
        !prepared.stream_path.exists(),
        "provider preparation must not pre-execute the application"
    );
    let source = fs::read_to_string(&prepared.source_yir_path).unwrap();
    let module = yir_syntax::parse_module(&source).unwrap();
    assert_eq!(module.application_sessions.len(), 2);
    assert_eq!(module.application_sessions[0].id, "image");
    let record = source
        .lines()
        .find(|line| line.starts_with("application-session "))
        .unwrap();
    let binary =
        crate::artifact_runtime_command::resolve_run_artifact_binary_path(&output.0).unwrap();
    let bytes = fs::read(binary).unwrap();
    assert!(
        bytes
            .windows(record.len())
            .any(|window| window == record.as_bytes()),
        "compiled host must retain the lifecycle registration in embedded YIR"
    );
    drop(bytes);
    let main_nodes = module
        .functions
        .iter()
        .filter(|function| function.role == YirFunctionRole::Entry)
        .flat_map(|function| function.body_nodes.iter().cloned())
        .collect::<BTreeSet<_>>();

    let socket_directory = Artifacts(std::env::temp_dir().join(format!(
        "ns-as-{}-{}",
        std::process::id(),
        NONCE.fetch_add(1, Ordering::Relaxed)
    )));
    DirBuilder::new()
        .mode(0o700)
        .create(&socket_directory.0)
        .unwrap();
    let socket_path = socket_directory.0.join("ipc");
    let listener = UnixListener::bind(&socket_path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let provider_output = output.0.clone();
    let worker = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(120)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(120)))
                        .unwrap();
                    return nsdb::serve_runtime_provider_session(&provider_output, &mut stream);
                }
                Err(error)
                    if error.kind() == ErrorKind::WouldBlock && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(5))
                }
                Err(error) => return Err(format!("application session accept failed: {error}")),
            }
        }
    });
    // The host supplies separate events. All image, application and receipt
    // policy still executes from compiled Nuis helpers, never from a Rust copy.
    let mut session = ApplicationEventPump::spawn(
        source.clone(),
        ApplicationProviderSource::Ipc(&socket_path),
        "image".to_owned(),
        configuration(),
    )
    .unwrap();
    let live = drive_images(&mut session, &main_nodes);
    drop(session);
    let served = worker.join().unwrap();
    let live = live.unwrap();
    assert_eq!(served.unwrap(), 2);
    assert_ne!(live[0], live[1]);
    for (frame, phase) in live.iter().zip([false, true]) {
        let input = image_input(phase);
        for (index, pixel) in frame.chunks_exact(4).enumerate() {
            let input_index = (index / 160 / 5 * 32 + index % 160 / 5) * 4;
            let source = &input[input_index..input_index + 4];
            assert_eq!(
                pixel,
                [255 - source[0], 255 - source[1], 255 - source[2], source[3]],
                "GPU pixel {index}, phase {phase}"
            );
        }
    }
    let payload = fs::read_to_string(
        output
            .0
            .join("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml"),
    )
    .unwrap();
    for (key, expected) in [
        ("runtime_dispatch_session_status", "verified"),
        ("runtime_dispatch_session_invocation_count", "2"),
        ("runtime_dispatch_session_worker_count", "1"),
        ("runtime_dispatch_session_lease_count", "1"),
        ("runtime_dispatch_session_request_sequences", "0,1"),
        (
            "runtime_dispatch_session_adapter_cache_statuses",
            "compiled,hit",
        ),
    ] {
        assert_eq!(toml_string_field(&payload, key), expected, "{key}");
    }
    assert!(payload.contains("metal.command-buffer.completed"));
    let stream = fs::read_to_string(&prepared.stream_path).unwrap();
    assert!(stream.contains("frame_count = 2"));
    let mut session = ApplicationEventPump::spawn(
        source.clone(),
        ApplicationProviderSource::Replay(&prepared.stream_path),
        "image".to_owned(),
        configuration(),
    )
    .unwrap();
    let replay = drive_images(&mut session, &main_nodes).unwrap();
    assert_eq!(
        live, replay,
        "persistent replay must match each live event, not just the final frame"
    );

    let mut session = ApplicationEventPump::spawn(
        source.clone(),
        ApplicationProviderSource::Replay(&prepared.stream_path),
        "image".to_owned(),
        configuration(),
    )
    .unwrap();
    receive(&mut session, ApplicationPumpOperation::Open)
        .trace
        .unwrap();
    session.event(vec![Value::Int(0)]).unwrap();
    receive(&mut session, ApplicationPumpOperation::Event)
        .trace
        .unwrap();
    session.close(vec![Value::Int(100)]).unwrap();
    let closed = receive(&mut session, ApplicationPumpOperation::Close);
    assert_eq!(closed.phase, ApplicationPumpPhase::Stopped);
    let error = closed.trace.unwrap_err();
    assert!(error.contains("unconsumed frame"), "{error}");
    assert_eq!(fs::read_to_string(&prepared.stream_path).unwrap(), stream);

    let drifted = source.replace("application-session image ", "application-session changed ");
    let error = with_registered_provider_application_session::<()>(
        &drifted,
        ApplicationProviderSource::Replay(&prepared.stream_path),
        "changed",
        configuration(),
        |_, _| panic!("drifted registration must fail before opening the application"),
    )
    .unwrap_err();
    assert!(error.contains("different YIR module"), "{error}");
}

fn drive_images(
    session: &mut ApplicationEventPump,
    main_nodes: &BTreeSet<String>,
) -> Result<Vec<Vec<u8>>, String> {
    let opened = receive(session, ApplicationPumpOperation::Open);
    assert_eq!(opened.phase, ApplicationPumpPhase::Open);
    assert!(opened.trace?.presented_frames.is_empty());
    assert_eq!(
        state_field(opened.state.as_ref().unwrap(), "frame_index"),
        0
    );
    let mut result = Vec::new();
    let mut previous_clock = 0;
    let mut previous_physical = 0;
    let mut previous_root = 0;
    for (ordinal, event) in [(1, 0), (2, 2)] {
        session.event(vec![Value::Int(event)])?;
        assert!(session
            .event(vec![Value::Int(event)])
            .unwrap_err()
            .contains("busy"));
        let reply = receive(session, ApplicationPumpOperation::Event);
        assert_eq!(reply.phase, ApplicationPumpPhase::Open);
        let trace = reply.trace?;
        let state = reply.state.as_ref().unwrap();
        assert_eq!(state_field(state, "frame_index"), ordinal);
        assert_eq!(state_field(state, "presented_frames"), ordinal);
        assert_eq!(state_field(state, "dropped_frames"), 0);
        let clock = state_field(state, "last_completion_tick");
        let root = state_field(state, "last_completion_root");
        assert!(clock > previous_clock);
        assert!(root > 0);
        if previous_root != 0 {
            assert_eq!(root, previous_root);
        }
        previous_clock = clock;
        previous_root = root;
        let fences = trace
            .provider_completion_witnesses
            .values()
            .filter(|witness| witness.clock_kind == ProviderCompletionClockKind::PhysicalFence)
            .collect::<Vec<_>>();
        assert_eq!(fences.len(), 1);
        let physical = fences[0].physical_source_clock.unwrap();
        assert!(physical > previous_physical);
        previous_physical = physical;
        assert_eq!(
            fences[0].physical_fence_source.as_deref(),
            Some("metal.command-buffer.completed")
        );
        assert_eq!(trace.presented_frames.len(), 1);
        let frame = &trace.presented_frames[0];
        assert_eq!((frame.width, frame.height), (160, 120));
        result.push(
            frame
                .rgba8
                .clone()
                .expect("real provider RGBA8, not reference rasterization"),
        );
        for step in trace.lane_steps.values().flatten() {
            assert!(
                !main_nodes.contains(step.rsplit_once(" -> ").unwrap().1),
                "main was replayed: {step}"
            );
        }
    }
    session.close(vec![Value::Int(previous_clock + 1)])?;
    let closed = receive(session, ApplicationPumpOperation::Close);
    assert_eq!(closed.phase, ApplicationPumpPhase::Closed);
    assert!(closed.trace?.presented_frames.is_empty());
    assert_eq!(state_field(closed.state.as_ref().unwrap(), "status"), 2);
    assert_eq!(
        state_field(closed.state.as_ref().unwrap(), "frame_index"),
        2
    );
    assert!(session.close(vec![Value::Int(999)]).is_err());
    assert!(session.poll()?.is_none());
    Ok(result)
}

fn receive(
    session: &mut ApplicationEventPump,
    operation: ApplicationPumpOperation,
) -> ApplicationPumpReply {
    let reply = session
        .wait(Duration::from_secs(130))
        .unwrap()
        .expect("bounded host event reply");
    assert_eq!(reply.operation, operation);
    assert_eq!(session.pending(), None);
    reply
}
