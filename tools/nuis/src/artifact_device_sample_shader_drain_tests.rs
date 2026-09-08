use super::*;
use crate::artifact_runtime_provider_results::{prepare_runtime_provider_results, transport};
use std::{
    io::Read, os::unix::net::UnixStream, path::PathBuf, process::Command, thread, time::Duration,
};
use yir_core::provider_runtime_ipc::{DispatchArguments, DispatchUpload, Message, SessionDrain};

#[path = "artifact_device_sample_shader_host_drain_tests.rs"]
mod host;

#[path = "artifact_device_sample_shader_packaged_drain_tests.rs"]
mod packaged;

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn arguments(phase: bool) -> DispatchArguments {
    let mut arguments = yir_domain_shader::ShaderDrawArguments {
        width: 160,
        height: 120,
        vertex_count: 3,
        instance_count: if phase { 3 } else { 1 },
    }
    .to_dispatch();
    yir_domain_shader::ShaderFragmentStorage {
        capability: yir_domain_shader::ShaderFragmentStorageCapability {
            slot: 3,
            element_count: 768,
        },
        upload: DispatchUpload::new("u32", vec![768], image_input(phase)).unwrap(),
    }
    .bind_dispatch(&mut arguments)
    .unwrap();
    arguments
}

#[test]
fn drains_registered_metal_session_without_replacing_success_evidence() {
    let output = Artifacts(temp_output_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    crate::handle_build(
        root.join("examples/projects/domains/ns_nova_image_showcase"),
        output.0.clone(),
        false,
        None,
        None,
        None,
    )
    .unwrap();
    let prepared = prepare_runtime_provider_results(&output.0)
        .unwrap()
        .unwrap();
    let binary =
        crate::artifact_runtime_command::resolve_run_artifact_binary_path(&output.0).unwrap();
    // Establish genuine completed evidence using the compiled Nuis application.
    let mut command = Command::new(&binary);
    command
        .arg("--export-frame")
        .arg(output.0.join("baseline.ppm"));
    let (status, count) = prepared
        .run_command_bounded(&mut command, Duration::from_secs(90))
        .unwrap();
    assert!(status.success());
    assert_eq!(count, 3);
    let previous = fs::read_dir(&output.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("nuis.runtime.provider-result")
                || path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("nuis.nsdb.provider-output")
                || path == &prepared.stream_path
        })
        .map(|path| {
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    assert!(previous
        .iter()
        .any(|(path, _)| path == &prepared.stream_path));

    for count in [0, 2] {
        let (mut client, mut server) = UnixStream::pair().unwrap();
        transport::configure_connection(&client, Duration::from_secs(60)).unwrap();
        transport::configure_connection(&server, Duration::from_secs(60)).unwrap();
        let provider_output = output.0.clone();
        let worker = thread::spawn(move || {
            nsdb::serve_runtime_provider_session_with_request_reader(
                &provider_output,
                &mut server,
                |stream| transport::read_request(stream, Duration::from_secs(10)),
            )
        });
        let Message::Hello(target) = Message::read_from(&mut client).unwrap() else {
            panic!("missing admission")
        };
        for sequence in 0..count {
            let phase = sequence == 1;
            let arguments = arguments(phase);
            Message::Dispatch {
                sequence,
                target: target.clone(),
                arguments: arguments.clone(),
            }
            .write_to(&mut client)
            .unwrap();
            let Message::Frame(frame) = Message::read_from(&mut client).unwrap() else {
                panic!("missing physical frame")
            };
            assert_eq!(frame.sequence, sequence);
            assert!(frame.arguments.matches_identity(&arguments).unwrap());
            assert_eq!(frame.shape, vec![160, 120]);
            assert_eq!(frame.payload.len(), 160 * 120 * 4);
            assert!(frame
                .completion_wire
                .contains("metal.command-buffer.completed"));
            let input = image_input(phase);
            for (index, pixel) in frame.payload.chunks_exact(4).enumerate() {
                let offset = (index / 160 / 5 * 32 + index % 160 / 5) * 4;
                let source = &input[offset..offset + 4];
                assert_eq!(
                    pixel,
                    [255 - source[0], 255 - source[1], 255 - source[2], source[3]]
                );
            }
        }
        let drain = SessionDrain {
            sequence: count,
            target,
        };
        if count > 0 {
            assert!(output.0.join(".nuis-provider-worker-image").is_dir());
        }
        Message::Drain(drain.clone()).write_to(&mut client).unwrap();
        assert_eq!(
            Message::read_from(&mut client).unwrap(),
            Message::Drained(drain.clone())
        );
        let outcome = worker.join().unwrap().unwrap();
        assert_eq!(outcome, nsdb::ProviderRuntimeSessionOutcome::Drained(drain));
        assert!(outcome.into_finished_count().is_err());
        assert!(!output.0.join(".nuis-provider-worker-image").exists());
        let mut byte = [0];
        assert_eq!(
            client.read(&mut byte).unwrap(),
            0,
            "drain must terminate the connection, not admit another event"
        );
        for (path, bytes) in &previous {
            assert_eq!(
                &fs::read(path).unwrap(),
                bytes,
                "drain replaced {}",
                path.display()
            );
        }
    }
    host::verify_host_cancellation(&output.0, &prepared.source_yir_path, &prepared.stream_path);
    packaged::verify_packaged_drain(&output.0, &binary, &prepared);
    for (path, bytes) in previous {
        assert_eq!(
            fs::read(&path).unwrap(),
            bytes,
            "host cancellation replaced {}",
            path.display()
        );
    }
}
