use super::*;
use std::thread;
use yir_core::{provider_runtime_ipc::DispatchFrame, Operation, ProviderPhysicalCompletion};

#[path = "provider_runtime_ipc_drain_tests.rs"]
mod drain_tests;

fn fixture() -> (DispatchTarget, Node, DispatchFrame) {
    let target = DispatchTarget {
        source_yir_fnv1a64: hash_bytes(b"source"),
        module: "shader".to_owned(),
        instruction: "draw_instanced".to_owned(),
        node: "draw".to_owned(),
        resource: "gpu".to_owned(),
    };
    let node = Node {
        name: target.node.clone(),
        resource: target.resource.clone(),
        op: Operation {
            module: target.module.clone(),
            instruction: target.instruction.clone(),
            args: Vec::new(),
        },
    };
    let frame = DispatchFrame {
        sequence: 0,
        arguments: DispatchArguments::parse("test.v1|count:u64:4").unwrap(),
        request_id: "render".to_owned(),
        provider_family: "test:device".to_owned(),
        element_type: "u8".to_owned(),
        layout: "image-2d-row-major:pixel-format=rgba8".to_owned(),
        shape: vec![1, 1],
        row_stride_bytes: 4,
        payload: vec![1, 2, 3, 255],
        completion_wire: ProviderPhysicalCompletion::new(
            "draw.clock",
            "test.clock",
            "test.fence",
            1,
        )
        .unwrap()
        .to_wire(),
    };
    (target, node, frame)
}

#[test]
fn live_client_requires_matching_sequence_layout_and_close_receipt() {
    for case in 0..5 {
        let (target, node, mut frame) = fixture();
        let arguments = frame.arguments.clone();
        let expected_arguments = arguments.clone();
        let (client, mut peer) = UnixStream::pair().unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let expected = target.clone();
        if case == 0 {
            frame.sequence = 1;
        }
        if case == 1 {
            frame.shape = vec![2, 1];
        }
        if case == 4 {
            frame.arguments.scalars.insert("count".to_owned(), 3);
        }
        let worker = thread::spawn(move || {
            assert_eq!(
                Message::read_from(&mut peer).unwrap(),
                Message::Dispatch {
                    sequence: 0,
                    target: expected,
                    arguments: expected_arguments,
                }
            );
            Message::Frame(frame).write_to(&mut peer).unwrap();
            if (2..4).contains(&case) {
                assert_eq!(Message::read_from(&mut peer).unwrap(), Message::Finish(1));
                Message::Closed(if case == 2 { 0 } else { 1 })
                    .write_to(&mut peer)
                    .unwrap();
            }
        });
        let mut client = ProviderRuntimeClient {
            stream: client,
            target,
            sequence: 0,
            frontier: true,
        };
        let result = client.take(&node, &arguments);
        if case < 2 || case == 4 {
            assert!(result.is_err());
            assert_eq!(client.sequence, 0);
        } else {
            assert!(result.is_ok());
            assert_eq!(client.finish().is_ok(), case == 3);
        }
        worker.join().unwrap();
    }
}

#[test]
fn live_client_rejects_disconnect_and_wrong_target() {
    let (target, node, frame) = fixture();
    let (stream, peer) = UnixStream::pair().unwrap();
    let mut client = ProviderRuntimeClient {
        stream,
        target,
        sequence: 0,
        frontier: true,
    };
    let wrong = Node {
        name: "other".to_owned(),
        ..node.clone()
    };
    assert!(client.take(&wrong, &frame.arguments).is_err());
    assert_eq!(client.sequence, 0);
    drop(peer);
    assert!(client.take(&node, &frame.arguments).is_err());
}

#[test]
fn typed_remote_rejections_are_admitted_before_category_delivery_and_never_advance() {
    for (phase, sequence, code, expected) in [
        (
            RejectionPhase::Dispatch,
            0,
            RejectionCode::Budget,
            ApplicationFailureKind::ProviderBudget,
        ),
        (
            RejectionPhase::Dispatch,
            0,
            RejectionCode::Execution,
            ApplicationFailureKind::ProviderExecution,
        ),
        (
            RejectionPhase::Dispatch,
            0,
            RejectionCode::Request,
            ApplicationFailureKind::ProviderRejected,
        ),
        (
            RejectionPhase::Receive,
            0,
            RejectionCode::Exchange,
            ApplicationFailureKind::ProviderExchange,
        ),
        (
            RejectionPhase::Dispatch,
            1,
            RejectionCode::Budget,
            ApplicationFailureKind::ProviderContract,
        ),
        (
            RejectionPhase::Finish,
            0,
            RejectionCode::Finalization,
            ApplicationFailureKind::ProviderContract,
        ),
    ] {
        let (target, node, frame) = fixture();
        let (stream, mut peer) = UnixStream::pair().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let worker = thread::spawn(move || {
            assert!(matches!(
                Message::read_from(&mut peer).unwrap(),
                Message::Dispatch { sequence: 0, .. }
            ));
            Message::Rejected(Rejection::new(
                phase,
                sequence,
                code,
                "device failed: budget exceeded",
            ))
            .write_to(&mut peer)
            .unwrap();
        });
        let mut client = ProviderRuntimeClient {
            stream,
            target,
            sequence: 0,
            frontier: true,
        };
        let error = client.take(&node, &frame.arguments).err().unwrap();
        assert_eq!(error.kind, expected);
        assert_eq!(client.sequence, 0);
        worker.join().unwrap();
    }
}

#[test]
fn finish_rejection_requires_finish_identity_and_cannot_become_closed() {
    for (phase, sequence, code, expected) in [
        (
            RejectionPhase::Finish,
            1,
            RejectionCode::Finalization,
            ApplicationFailureKind::ProviderFinalization,
        ),
        (
            RejectionPhase::Receive,
            1,
            RejectionCode::Exchange,
            ApplicationFailureKind::ProviderExchange,
        ),
        (
            RejectionPhase::Dispatch,
            1,
            RejectionCode::Execution,
            ApplicationFailureKind::ProviderContract,
        ),
        (
            RejectionPhase::Drain,
            1,
            RejectionCode::Finalization,
            ApplicationFailureKind::ProviderContract,
        ),
        (
            RejectionPhase::Finish,
            0,
            RejectionCode::Finalization,
            ApplicationFailureKind::ProviderContract,
        ),
    ] {
        let (target, _, _) = fixture();
        let (stream, mut peer) = UnixStream::pair().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let worker = thread::spawn(move || {
            assert_eq!(Message::read_from(&mut peer).unwrap(), Message::Finish(1));
            Message::Rejected(Rejection::new(phase, sequence, code, "finish failed"))
                .write_to(&mut peer)
                .unwrap();
        });
        let mut client = ProviderRuntimeClient {
            stream,
            target,
            sequence: 1,
            frontier: true,
        };
        assert_eq!(client.finish().unwrap_err().kind, expected);
        assert_eq!(client.sequence, 1);
        worker.join().unwrap();
    }
}

#[test]
fn finish_cannot_consume_a_provider_drain_receipt_as_completion() {
    let (target, _, _) = fixture();
    let drain = yir_core::provider_runtime_ipc::SessionDrain {
        sequence: 1,
        target: target.clone(),
    };
    let (stream, mut peer) = UnixStream::pair().unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let worker = thread::spawn(move || {
        assert_eq!(Message::read_from(&mut peer).unwrap(), Message::Finish(1));
        Message::Drained(drain).write_to(&mut peer).unwrap();
    });
    let mut client = ProviderRuntimeClient {
        stream,
        target,
        sequence: 1,
        frontier: true,
    };
    assert_eq!(
        client.finish().unwrap_err().kind,
        ApplicationFailureKind::ProviderContract
    );
    assert_eq!(client.sequence, 1);
    worker.join().unwrap();
}
