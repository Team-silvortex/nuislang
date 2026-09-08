use super::*;
use std::{
    fs::DirBuilder,
    io::ErrorKind,
    os::unix::{fs::DirBuilderExt, net::UnixListener},
    time::Instant,
};
use yir_runtime_host::{
    ApplicationProviderSource, ApplicationPumpPhase, ProviderDrainObservation, WindowSession,
    WindowSessionReply,
};

pub(super) fn verify_host_cancellation(output: &Path, source: &Path, replay: &Path) {
    let source = fs::read_to_string(source).unwrap();
    let socket_directory = Artifacts(std::env::temp_dir().join(format!(
        "ns-hd-{}-{}",
        std::process::id(),
        NONCE.fetch_add(1, Ordering::Relaxed),
    )));
    DirBuilder::new()
        .mode(0o700)
        .create(&socket_directory.0)
        .unwrap();
    for count in [0, 2] {
        let socket_path = socket_directory.0.join(format!("ipc-{count}"));
        let listener = UnixListener::bind(&socket_path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let provider_output = output.to_owned();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(30);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error)
                        if error.kind() == ErrorKind::WouldBlock && Instant::now() < deadline =>
                    {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => return Err(format!("drain test accept failed: {error}")),
                }
            };
            transport::configure_connection(&stream, Duration::from_secs(60)).unwrap();
            nsdb::serve_runtime_provider_session_with_request_reader(
                &provider_output,
                &mut stream,
                |stream| transport::read_request(stream, Duration::from_secs(10)),
            )
        });
        // Compiled Nuis owns the callbacks, image state and storage upload.
        // This host adapter does not substitute a Rust application implementation.
        let mut session = WindowSession::spawn(
            source.clone(),
            ApplicationProviderSource::Ipc(&socket_path),
            "window".to_owned(),
            640,
            400,
        )
        .unwrap();
        assert!(receive(&mut session).frame.unwrap().is_none());
        for event in 0..count {
            session
                .event(
                    if event == 0 { 0 } else { 1 },
                    if event == 0 { 0 } else { 32 },
                )
                .unwrap();
            let frame = receive(&mut session).frame.unwrap().unwrap();
            let header = b"P6\n160 120\n255\n";
            assert_eq!(&frame[..header.len()], header);
            assert_eq!(frame.len(), header.len() + 160 * 120 * 3);
            let input = image_input(event == 1);
            for (index, pixel) in frame[header.len()..].chunks_exact(3).enumerate() {
                let offset = (index / 160 / 5 * 32 + index % 160 / 5) * 4;
                let source = &input[offset..offset + 3];
                assert_eq!(pixel, [255 - source[0], 255 - source[1], 255 - source[2]]);
            }
        }
        if count > 0 {
            assert!(output.join(".nuis-provider-worker-image").exists());
        }
        let mut ticket = session.cancel_with_provider_drain().unwrap();
        assert_eq!(session.phase(), ApplicationPumpPhase::Stopped);
        assert!(session.outcome().is_none());
        assert!(session.take_outcome_delivery().is_none());
        drop(session);
        let ack = ticket
            .wait(Duration::from_secs(90))
            .unwrap()
            .expect("host retirement");
        assert!(!ack.cleanup_completed());
        assert_eq!(ack.failure_kind(), yir_core::ApplicationFailureKind::None);
        assert_eq!(
            ack.provider_drain(),
            ProviderDrainObservation::Drained {
                completed_dispatches: count
            }
        );
        assert!(!output.join(".nuis-provider-worker-image").exists());
        assert!(ticket.poll().unwrap().is_none());
        let outcome = worker.join().unwrap().unwrap();
        assert!(
            matches!(&outcome, nsdb::ProviderRuntimeSessionOutcome::Drained(drain)
            if drain.sequence == count)
        );
        assert!(outcome.into_finished_count().is_err());
    }

    let previous = fs::read(replay).unwrap();
    let mut session = WindowSession::spawn(
        source,
        ApplicationProviderSource::Replay(replay),
        "window".to_owned(),
        640,
        400,
    )
    .unwrap();
    receive(&mut session).frame.unwrap();
    session.event(0, 0).unwrap();
    assert!(receive(&mut session).frame.unwrap().is_some());
    let mut ticket = session.cancel_with_provider_drain().unwrap();
    assert!(session.outcome().is_none());
    drop(session);
    let ack = ticket.wait(Duration::from_secs(10)).unwrap().unwrap();
    assert_eq!(ack.provider_drain(), ProviderDrainObservation::ReplayOnly);
    assert!(!ack.cleanup_completed());
    assert_eq!(fs::read(replay).unwrap(), previous);
}

fn receive(session: &mut WindowSession) -> WindowSessionReply {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        if let Some(reply) = session.poll().unwrap() {
            return reply;
        }
        assert!(Instant::now() < deadline, "host frame deadline");
        thread::sleep(Duration::from_millis(5));
    }
}
