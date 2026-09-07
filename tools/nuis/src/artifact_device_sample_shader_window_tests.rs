use super::*;
use std::process::{Command, Stdio};

#[path = "artifact_device_sample_shader_failure_tests.rs"]
mod failure;

#[test]
fn compiled_window_routes_appkit_events_through_registered_nuis_and_live_metal() {
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
    .expect("build compiled window artifact");
    let prepared =
        crate::artifact_runtime_provider_results::prepare_runtime_provider_results(&output.0)
            .unwrap()
            .unwrap();
    let binary =
        crate::artifact_runtime_command::resolve_run_artifact_binary_path(&output.0).unwrap();
    let log_path = output.0.join("window.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut command = Command::new(&binary);
    command
        .args(["--window-session", "window", "--window-events", "32,128578"])
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log));
    // A wedged AppKit loop must not strand this test or the user's desktop.
    let result = prepared.run_command_bounded(&mut command, Duration::from_secs(60));
    let log = fs::read_to_string(&log_path).unwrap();
    let (status, invocations) = result.unwrap_or_else(|error| panic!("{error}\n{log}"));
    assert!(status.success(), "{status}\n{log}");
    assert_eq!(
        invocations, 2,
        "main/timer must not replay the whole module\n{log}"
    );
    assert_eq!(log.matches("window_session_opened").count(), 1, "{log}");
    assert_eq!(log.matches("window_session_presented").count(), 2, "{log}");
    assert_eq!(log.matches("window_session_closed").count(), 1, "{log}");
    assert_eq!(
        log.matches("window_session_close_requested").count(),
        1,
        "{log}"
    );
    assert!(log.contains("window_session_key=32\n"), "{log}");
    assert!(log.contains("window_session_key=128578\n"), "{log}");
    assert!(log.contains("window_session_close_reason=0\n"), "{log}");
    assert!(
        log.contains("window_session_close_failure_kind=0\n"),
        "{log}"
    );
    assert!(log.contains("window_session_failure_kind=0\n"), "{log}");
    assert!(
        log.contains("window_session_cleanup_completed=1\n"),
        "{log}"
    );
    assert!(!log.contains("embedded runtime frame generation"), "{log}");
    assert!(!log.contains("while busy"), "{log}");
    let source = fs::read_to_string(&prepared.source_yir_path).unwrap();
    let payload = fs::read_to_string(
        output
            .0
            .join("nuis.nsdb.provider-output.metal-apple-silicon-gpu.toml"),
    )
    .unwrap();
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_worker_count"),
        "1"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_request_sequences"),
        "0,1"
    );
    assert_eq!(
        toml_string_field(&payload, "runtime_dispatch_session_adapter_cache_statuses"),
        "compiled,hit"
    );
    assert!(payload.contains("metal.command-buffer.completed"));
    // Replay the event script by the same registered Nuis helpers, checking every
    // pixel saved by the compiled window process rather than a reference renderer.
    with_registered_provider_application_session(
        &source,
        ApplicationProviderSource::Replay(&prepared.stream_path),
        "window",
        vec![Value::Int(640), Value::Int(400)],
        |session, opened| {
            assert!(opened.presented_frames.is_empty());
            for ((kind, code), phase) in [((0, 0), false), ((1, 32), true)] {
                let trace = session.event(vec![Value::Int(kind), Value::Int(code)])?;
                assert_eq!(trace.presented_frames.len(), 1);
                let pixels = trace.presented_frames[0].rgba8.as_ref().unwrap();
                let input = image_input(phase);
                for (index, pixel) in pixels.chunks_exact(4).enumerate() {
                    let source_index = (index / 160 / 5 * 32 + index % 160 / 5) * 4;
                    let source = &input[source_index..source_index + 4];
                    assert_eq!(
                        pixel,
                        [255 - source[0], 255 - source[1], 255 - source[2], source[3]]
                    );
                }
            }
            let trace = session.event(vec![Value::Int(1), Value::Int(0x1f642)])?;
            assert!(
                trace.presented_frames.is_empty(),
                "unbound key must not redraw"
            );
            assert!(session
                .close(vec![Value::Int(0), Value::Int(0)])?
                .unwrap()
                .presented_frames
                .is_empty());
            Ok(())
        },
    )
    .unwrap();
    // Wrong host profile and malformed Unicode scripts fail without opening a
    // provider connection or substituting the old preview path.
    for args in [
        vec!["--window-session", "image", "--window-events", ""],
        vec!["--window-session", "window", "--window-events", "55296"],
    ] {
        let mut child = Command::new(&binary)
            .args(args)
            .env_remove(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV)
            .env(
                yir_runtime_host::PROVIDER_RESULT_STREAM_ENV,
                &prepared.stream_path,
            )
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(!status.success());
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("rejected window launch did not exit before its deadline");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    crate::artifact_runtime_command::handle_run_artifact_with_window(
        output.0.clone(),
        crate::cli::WindowSessionOptions {
            id: "window".to_owned(),
            events: Some("32,128578".to_owned()),
        },
    )
    .expect("production run-artifact window frontdoor");
    failure::verify_compiled_provider_failure_cleanup(&output.0, &binary);
}
