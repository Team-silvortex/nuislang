use super::*;

#[test]
fn framed_ipc_rejects_corruption_truncation_and_unbounded_allocation() {
    let frame = Message::Frame(DispatchFrame {
        sequence: 2,
        arguments: DispatchArguments::parse("test.v1|count:u64:4").unwrap(),
        request_id: "draw".to_owned(),
        provider_family: "test:device".to_owned(),
        element_type: "u8".to_owned(),
        layout: "bytes".to_owned(),
        shape: vec![4],
        row_stride_bytes: 4,
        payload: vec![1, 2, 3, 4],
        completion_wire: crate::ProviderPhysicalCompletion::new(
            "draw.clock",
            "test.clock",
            "test.fence",
            1,
        )
        .unwrap()
        .to_wire(),
    });
    let mut wire = Vec::new();
    frame.write_to(&mut wire).unwrap();
    assert_eq!(Message::read_from(&mut wire.as_slice()).unwrap(), frame);
    for end in [0, 3, wire.len() - 1] {
        assert!(Message::read_from(&mut &wire[..end]).is_err());
    }
    let mut corrupted = wire.clone();
    *corrupted.last_mut().unwrap() ^= 1;
    assert!(Message::read_from(&mut corrupted.as_slice())
        .unwrap_err()
        .contains("identity mismatch"));
    assert!(Message::read_from(&mut u32::MAX.to_le_bytes().as_slice())
        .unwrap_err()
        .contains("header size"));

    let size = u32::from_le_bytes(wire[..4].try_into().unwrap()) as usize;
    let header = String::from_utf8(wire[4..4 + size].to_vec()).unwrap();
    let oversized = header.replace("\n4\n0x", "\n16777217\n0x");
    let mut input = (oversized.len() as u32).to_le_bytes().to_vec();
    input.extend_from_slice(oversized.as_bytes());
    assert!(Message::read_from(&mut input.as_slice())
        .unwrap_err()
        .contains("payload size"));
}

#[test]
fn target_admission_rejects_protocol_and_field_injection() {
    let target = DispatchTarget {
        source_yir_fnv1a64: hash_bytes(b"yir"),
        module: "shader".to_owned(),
        instruction: "draw_instanced".to_owned(),
        node: "draw.first".to_owned(),
        resource: "gpu".to_owned(),
    };
    let mut wire = Vec::new();
    Message::Hello(target.clone()).write_to(&mut wire).unwrap();
    assert_eq!(
        Message::read_from(&mut wire.as_slice()).unwrap(),
        Message::Hello(target.clone())
    );
    wire[4] ^= 1;
    assert!(Message::read_from(&mut wire.as_slice()).is_err());
    let injected = DispatchTarget {
        node: "draw\nfinish".to_owned(),
        ..target
    };
    assert!(Message::Dispatch {
        sequence: 0,
        target: injected,
        arguments: DispatchArguments::parse("test.v1|count:u64:4").unwrap(),
    }
    .write_to(&mut Vec::new())
    .is_err());
}
