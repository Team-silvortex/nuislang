use super::*;
use std::{cell::Cell, io::Cursor};

struct Duplex {
    input: Cursor<Vec<u8>>,
    output: Vec<u8>,
}

impl Read for Duplex {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.input.read(bytes)
    }
}

impl Write for Duplex {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.output.write(bytes)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn target() -> DispatchTarget {
    DispatchTarget {
        source_yir_fnv1a64: yir_core::provider_runtime_ipc::hash_bytes(b"source"),
        module: "shader".to_owned(),
        instruction: "draw_instanced".to_owned(),
        node: "draw.first".to_owned(),
        resource: "gpu".to_owned(),
    }
}

fn drain(sequence: usize) -> SessionDrain {
    SessionDrain {
        sequence,
        target: target(),
    }
}

fn arguments() -> DispatchArguments {
    DispatchArguments::parse("test.v1|count:u64:4").unwrap()
}

fn dispatch(sequence: usize) -> Message {
    Message::Dispatch {
        sequence,
        target: target(),
        arguments: arguments(),
    }
}

fn wire(messages: &[Message]) -> Duplex {
    let mut input = Vec::new();
    for message in messages {
        message.write_to(&mut input).unwrap();
    }
    Duplex {
        input: Cursor::new(input),
        output: Vec::new(),
    }
}

fn outputs() -> NativeProviderOutputs {
    let mut frame = crate::provider_runtime_result_stream_tests::result();
    let target = target();
    frame.source_yir_fnv1a64 = target.source_yir_fnv1a64;
    frame.node = target.node;
    let mut output = NativeProviderOutputs::empty();
    output.runtime_results.push(frame);
    output
}

#[test]
fn drain_is_terminal_at_zero_midstream_and_exact_dispatch_limit() {
    for count in [0, 2, MAX_DISPATCHES] {
        let mut requests = (0..count).map(dispatch).collect::<Vec<_>>();
        requests.push(Message::Drain(drain(count)));
        // Even already-buffered work is not admitted after a terminal drain.
        requests.push(Message::Finish(count));
        let mut stream = wire(&requests);
        let mut executed = 0;
        let end = dispatch_loop(&mut stream, &target(), 4, Message::read_from, |_| {
            executed += 1;
            Ok(outputs())
        })
        .unwrap();
        assert_eq!(executed, count);
        assert!(matches!(&end, DispatchEnd::Drain(receipt) if receipt == &drain(count)));
        assert_eq!(
            Message::read_from(&mut stream.input).unwrap(),
            Message::Finish(count)
        );
        let mut frames = stream.output.as_slice();
        for sequence in 0..count {
            assert!(
                matches!(Message::read_from(&mut frames).unwrap(), Message::Frame(frame) if frame.sequence == sequence)
            );
        }
        assert!(
            frames.is_empty(),
            "dispatch loop must not acknowledge drain before close"
        );
        let mut reply = Vec::new();
        let outcome = finalize_execution(
            &mut reply,
            Ok(end),
            || Ok(()),
            |_, _| panic!("drain must not persist results"),
        )
        .unwrap();
        assert_eq!(
            outcome,
            ProviderRuntimeSessionOutcome::Drained(drain(count))
        );
        assert!(outcome.into_finished_count().is_err());
        let mut reply = reply.as_slice();
        assert_eq!(
            Message::read_from(&mut reply).unwrap(),
            Message::Drained(drain(count))
        );
        assert!(reply.is_empty(), "one acknowledgement only");
    }
}

#[test]
fn drain_acknowledgement_waits_for_successful_provider_close() {
    struct AckWriter<'a> {
        closed: &'a Cell<bool>,
        bytes: Vec<u8>,
    }
    impl Write for AckWriter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            assert!(self.closed.get(), "acknowledged before provider retirement");
            self.bytes.write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let closed = Cell::new(false);
    let mut writer = AckWriter {
        closed: &closed,
        bytes: Vec::new(),
    };
    finalize_execution(
        &mut writer,
        Ok(DispatchEnd::Drain(drain(0))),
        || {
            assert!(!closed.replace(true), "closed more than once");
            Ok(())
        },
        |_, _| panic!("drain published replay"),
    )
    .unwrap();
    assert!(closed.get());
    assert_eq!(
        Message::read_from(&mut writer.bytes.as_slice()).unwrap(),
        Message::Drained(drain(0))
    );
}

#[test]
fn drain_cleanup_or_ack_write_failure_cannot_claim_retirement() {
    let mut output = Vec::new();
    let error = finalize_execution(
        &mut output,
        Ok(DispatchEnd::Drain(drain(2))),
        || Err("worker still active".to_owned()),
        |_, _| panic!("drain published"),
    )
    .unwrap_err();
    assert_eq!(error.phase, RejectionPhase::Drain);
    assert_eq!(error.sequence, 2);
    assert_eq!(error.code, RejectionCode::Finalization);
    assert!(output.is_empty());

    struct FailedWriter;
    impl Write for FailedWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let error = finalize_execution(
        &mut FailedWriter,
        Ok(DispatchEnd::Drain(drain(2))),
        || Ok(()),
        |_, _| panic!("drain published"),
    )
    .unwrap_err();
    assert_eq!(error.phase, RejectionPhase::Drain);
    assert_eq!(error.code, RejectionCode::Exchange);
    assert_eq!(error.sequence, 2);
}

#[test]
fn mismatched_or_unsolicited_drain_is_not_admitted() {
    let mut wrong = drain(0);
    wrong.target.source_yir_fnv1a64 = yir_core::provider_runtime_ipc::hash_bytes(b"different");
    for message in [
        Message::Drain(wrong),
        Message::Drain(drain(1)),
        Message::Drained(drain(0)),
    ] {
        let mut stream = wire(&[message]);
        let execution = dispatch_loop(&mut stream, &target(), 4, Message::read_from, |_| {
            panic!("invalid drain dispatched")
        });
        let closed = Cell::new(false);
        let error = finalize_execution(
            &mut stream,
            execution,
            || {
                closed.set(true);
                Ok(())
            },
            |_, _| panic!("invalid drain persisted"),
        )
        .unwrap_err();
        assert!(closed.get());
        assert_eq!(error.phase, RejectionPhase::Receive);
        assert_eq!(error.code, RejectionCode::Request);
        assert!(stream.output.is_empty());
    }
}

#[test]
fn disconnect_and_first_failure_are_not_reclassified_as_drain() {
    let mut stream = wire(&[]);
    let execution = dispatch_loop(&mut stream, &target(), 4, Message::read_from, |_| {
        panic!("disconnect dispatched")
    });
    let error = finalize_execution(
        &mut stream,
        execution,
        || Err("close also failed".to_owned()),
        |_, _| panic!("disconnect published"),
    )
    .unwrap_err();
    assert_eq!(error.phase, RejectionPhase::Receive);
    assert_eq!(error.code, RejectionCode::Exchange);
    assert!(error.detail.contains("provider cleanup failed"));
    assert!(stream.output.is_empty());
    assert_eq!(
        ProviderRuntimeSessionOutcome::Finished(2)
            .into_finished_count()
            .unwrap(),
        2
    );
}
