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
        let result = dispatch_loop(&mut Cursor::new(wire), &target, |_| {
            panic!("invalid event executed device")
        });
        assert!(result.is_err());
    }
    assert!(
        dispatch_loop(&mut Cursor::new(Vec::<u8>::new()), &target, |_| panic!(
            "disconnected peer executed device"
        ))
        .is_err()
    );
}

#[test]
fn zero_draw_lifecycle_closes_without_invoking_provider() {
    let mut wire = Vec::new();
    Message::Finish(0).write_to(&mut wire).unwrap();
    let (count, output) = dispatch_loop(&mut Cursor::new(wire), &target(), |_| {
        panic!("skipped draw executed device")
    })
    .unwrap();
    assert_eq!(count, 0);
    assert!(output.runtime_results.is_empty());
    assert!(output.runtime_session_evidence.is_none());
}
