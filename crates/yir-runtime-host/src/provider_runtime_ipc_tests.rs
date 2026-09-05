use super::*;
use std::thread;
use yir_core::{provider_runtime_ipc::DispatchFrame, Operation, ProviderPhysicalCompletion};

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
    for case in 0..4 {
        let (target, node, mut frame) = fixture();
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
        let worker = thread::spawn(move || {
            assert_eq!(
                Message::read_from(&mut peer).unwrap(),
                Message::Dispatch {
                    sequence: 0,
                    target: expected
                }
            );
            Message::Frame(frame).write_to(&mut peer).unwrap();
            if case >= 2 {
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
        };
        let result = client.take(&node);
        if case < 2 {
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
    let (target, node, _) = fixture();
    let (stream, peer) = UnixStream::pair().unwrap();
    let mut client = ProviderRuntimeClient {
        stream,
        target,
        sequence: 0,
    };
    let wrong = Node {
        name: "other".to_owned(),
        ..node.clone()
    };
    assert!(client.take(&wrong).is_err());
    assert_eq!(client.sequence, 0);
    drop(peer);
    assert!(client.take(&node).is_err());
}
