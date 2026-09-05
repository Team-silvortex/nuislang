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

#[test]
fn dispatch_roundtrips_owned_immutable_resources_without_new_carrier_authority() {
    let mut arguments = DispatchArguments::parse("test.v1|count:u64:4").unwrap();
    arguments.resources.insert(
        "input.2".to_owned(),
        DispatchResource {
            element_type: "f32".to_owned(),
            shape: vec![4],
            bytes: vec![0; 16],
        },
    );
    let message = Message::Dispatch {
        sequence: 0,
        target: DispatchTarget {
            source_yir_fnv1a64: hash_bytes(b"source"),
            module: "test".to_owned(),
            instruction: "draw".to_owned(),
            node: "frame".to_owned(),
            resource: "device".to_owned(),
        },
        arguments,
    };
    let mut wire = Vec::new();
    message.write_to(&mut wire).unwrap();
    assert_eq!(Message::read_from(&mut wire.as_slice()).unwrap(), message);
    *wire.last_mut().unwrap() = b'1';
    assert!(Message::read_from(&mut wire.as_slice())
        .unwrap_err()
        .contains("identity mismatch"));
}

#[test]
fn bulk_upload_is_framed_outside_control_header_and_identity_survives_descriptor_only_reply() {
    let bytes: Vec<u8> = (0..4096).map(|index| (index % 251) as u8).collect();
    let mut arguments = DispatchArguments::parse("test.v1|count:u64:1024").unwrap();
    arguments.uploads.insert(
        "input.0".to_owned(),
        DispatchUpload::new("u32", vec![1024], bytes.clone()).unwrap(),
    );
    let control = arguments.to_wire().unwrap();
    assert!(control.len() <= 256);
    let descriptor = DispatchArguments::parse(&control).unwrap();
    assert!(descriptor.uploads["input.0"].payload().is_err());
    assert!(descriptor.matches_identity(&arguments).unwrap());
    let target = DispatchTarget {
        source_yir_fnv1a64: hash_bytes(b"source"),
        module: "example".to_owned(),
        instruction: "dispatch".to_owned(),
        node: "node".to_owned(),
        resource: "device".to_owned(),
    };
    let message = Message::Dispatch {
        sequence: 0,
        target: target.clone(),
        arguments: arguments.clone(),
    };
    let mut wire = Vec::new();
    message.write_to(&mut wire).unwrap();
    let header_size = u32::from_le_bytes(wire[..4].try_into().unwrap()) as usize;
    assert_eq!(&wire[4 + header_size..], bytes);
    assert!(Message::read_from(&mut wire.as_slice()).unwrap() == message);
    let mut truncated = &wire[..wire.len() - 1];
    assert!(Message::read_from(&mut truncated)
        .unwrap_err()
        .contains("read failed"));
    let mut corrupted = wire.clone();
    *corrupted.last_mut().unwrap() ^= 1;
    assert!(Message::read_from(&mut corrupted.as_slice())
        .unwrap_err()
        .contains("identity mismatch"));
    let mut output = Vec::new();
    assert!(Message::Dispatch {
        sequence: 0,
        target,
        arguments: descriptor
    }
    .write_to(&mut output)
    .unwrap_err()
    .contains("payload is missing"));
    assert!(
        output.is_empty(),
        "validate upload before writing any request bytes"
    );
    let header = std::str::from_utf8(&wire[4..4 + header_size]).unwrap();
    for invalid in [
        header.replace("u32:1024:4096", "u32:4194305:16777220"),
        header.replace("u32:1024:4096", "u32:1024:4095"),
        header.replace("u32:1024:4096", "u32:01024:4096"),
        header.replace(CONTRACT, "nuis-yir-provider-runtime-ipc-v2"),
    ] {
        let mut packet = (invalid.len() as u32).to_le_bytes().to_vec();
        packet.extend_from_slice(invalid.as_bytes());
        assert!(Message::read_from(&mut packet.as_slice()).is_err());
    }
    let mut changed = arguments.clone();
    changed.uploads.insert(
        "input.0".to_owned(),
        DispatchUpload::new("f32", vec![1024], bytes).unwrap(),
    );
    assert!(!changed.matches_identity(&arguments).unwrap());
    changed.uploads.insert(
        "input.0".to_owned(),
        DispatchUpload::new("u32", vec![1024], vec![0; 4096]).unwrap(),
    );
    assert!(!changed.matches_identity(&arguments).unwrap());
}

#[test]
fn upload_descriptor_rejects_alias_names_shape_overflow_and_total_budget() {
    let mut arguments = DispatchArguments::parse("test.v1|count:u64:1").unwrap();
    let upload = DispatchUpload::new("u8", vec![4], vec![1; 4]).unwrap();
    arguments.uploads.insert("count".to_owned(), upload.clone());
    assert!(arguments.to_wire().unwrap_err().contains("duplicated"));
    for shape in [vec![], vec![0], vec![1; 5], vec![usize::MAX, 2]] {
        let mut invalid = upload.clone();
        invalid.shape = shape;
        assert!(invalid.validate().is_err());
    }
    let hash = hash_bytes(&[0]);
    let wire = format!("test.v1|count:u64:1|a:immutable-upload-le:u8:16777216:16777216:{hash}|b:immutable-upload-le:u8:1:1:{hash}");
    assert!(DispatchArguments::parse(&wire)
        .unwrap_err()
        .contains("total exceeds"));
}
