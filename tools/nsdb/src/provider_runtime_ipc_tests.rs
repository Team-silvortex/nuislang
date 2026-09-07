use super::*;
use std::io::Cursor;

fn target() -> DispatchTarget {
    DispatchTarget {
        source_yir_fnv1a64: yir_core::provider_runtime_ipc::hash_bytes(b"test"),
        module: "shader".to_owned(),
        instruction: "draw_instanced".to_owned(),
        node: "draw.first".to_owned(),
        resource: "gpu".to_owned(),
    }
}

#[test]
fn invalid_events_and_disconnect_never_reach_device_execution() {
    let target = target();
    let wrong = DispatchTarget {
        node: "draw.other".to_owned(),
        ..target.clone()
    };
    for event in [
        Message::Dispatch {
            sequence: 1,
            target: target.clone(),
            arguments: DispatchArguments::parse("test.v1|count:u64:4").unwrap(),
        },
        Message::Dispatch {
            sequence: 0,
            target: wrong,
            arguments: DispatchArguments::parse("test.v1|count:u64:4").unwrap(),
        },
        Message::Finish(1),
        Message::Closed(0),
    ] {
        let mut wire = Vec::new();
        event.write_to(&mut wire).unwrap();
        let result = dispatch_loop(
            &mut Cursor::new(wire),
            &target,
            4,
            Message::read_from,
            |_| panic!("invalid event executed device"),
        );
        assert!(result.is_err());
    }
    assert!(dispatch_loop(
        &mut Cursor::new(Vec::<u8>::new()),
        &target,
        4,
        Message::read_from,
        |_| panic!("disconnected peer executed device")
    )
    .is_err());
}

#[test]
fn zero_draw_lifecycle_closes_without_invoking_provider() {
    let mut wire = Vec::new();
    Message::Finish(0).write_to(&mut wire).unwrap();
    let (count, output) = dispatch_loop(
        &mut Cursor::new(wire),
        &target(),
        4,
        Message::read_from,
        |_| panic!("skipped draw executed device"),
    )
    .unwrap();
    assert_eq!(count, 0);
    assert!(output.runtime_results.is_empty());
    assert!(output.runtime_session_evidence.is_none());
}

#[test]
fn exhausted_replay_budget_rejects_before_device_execution() {
    use yir_core::provider_runtime_ipc::{MAX_PAYLOAD_BYTES, MAX_REPLAY_BYTES};
    let mut budget = ReplayBudget::default();
    for _ in 0..MAX_REPLAY_BYTES / MAX_PAYLOAD_BYTES {
        budget.reserve(MAX_PAYLOAD_BYTES).unwrap();
    }
    let full = budget.clone();
    let error = execute_reserved(&mut budget, 4, || {
        panic!("budget exhaustion must not reach device execution")
    })
    .err()
    .unwrap();
    assert!(error.contains("storage budget"));
    assert_eq!(budget, full);
}

#[test]
fn failed_admitted_device_work_does_not_refund_its_reservation() {
    let mut budget = ReplayBudget::default();
    let error = execute_reserved(&mut budget, 4, || Err("device failed".to_owned()))
        .err()
        .unwrap();
    assert_eq!(error, "device failed");
    let mut reserved = ReplayBudget::default();
    reserved.reserve(4).unwrap();
    assert_eq!(budget, reserved);
    assert!(execute_reserved(&mut budget, 4, || Ok(NativeProviderOutputs::empty())).is_err());
    reserved.reserve(4).unwrap();
    assert_eq!(budget, reserved);
}

#[test]
fn device_result_must_fit_the_registered_reservation() {
    let mut budget = ReplayBudget::default();
    let error = execute_reserved(&mut budget, 1, || {
        let mut output = NativeProviderOutputs::empty();
        output
            .runtime_results
            .push(crate::provider_runtime_result_stream_tests::result());
        Ok(output)
    })
    .err()
    .unwrap();
    assert!(error.contains("reserved output extent"));
}

#[test]
fn exact_dispatch_limit_can_still_finish_without_resetting_the_session() {
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
    let target = target();
    let arguments = DispatchArguments::parse("test.v1|count:u64:4").unwrap();
    let mut input = Vec::new();
    for sequence in 0..MAX_DISPATCHES {
        Message::Dispatch {
            sequence,
            target: target.clone(),
            arguments: arguments.clone(),
        }
        .write_to(&mut input)
        .unwrap();
    }
    Message::Finish(MAX_DISPATCHES)
        .write_to(&mut input)
        .unwrap();
    let mut stream = Duplex {
        input: Cursor::new(input),
        output: Vec::new(),
    };
    let mut executed = 0;
    let (count, outputs) = dispatch_loop(&mut stream, &target, 4, Message::read_from, |_| {
        executed += 1;
        let mut frame = crate::provider_runtime_result_stream_tests::result();
        frame.source_yir_fnv1a64 = target.source_yir_fnv1a64.clone();
        frame.node = target.node.clone();
        let mut output = NativeProviderOutputs::empty();
        output.runtime_results.push(frame);
        Ok(output)
    })
    .unwrap();
    assert_eq!(executed, MAX_DISPATCHES);
    assert_eq!(count, MAX_DISPATCHES);
    assert_eq!(outputs.runtime_results.len(), MAX_DISPATCHES);
    let mut replies = Cursor::new(stream.output);
    for sequence in 0..MAX_DISPATCHES {
        let Message::Frame(frame) = Message::read_from(&mut replies).unwrap() else {
            panic!("expected ordered frame")
        };
        assert_eq!(frame.sequence, sequence);
    }
    assert_eq!(replies.position() as usize, replies.get_ref().len());
}

#[test]
fn request_reader_failure_is_terminal_without_retry_or_device_execution() {
    let mut reads = 0;
    let error = dispatch_loop(
        &mut Cursor::new(Vec::<u8>::new()),
        &target(),
        4,
        |_| {
            reads += 1;
            Err("request deadline exceeded after partial input".to_owned())
        },
        |_| panic!("failed request reader reached device execution"),
    )
    .err()
    .unwrap();
    assert_eq!(reads, 1);
    assert_eq!(error, "request deadline exceeded after partial input");
}
