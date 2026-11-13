use super::*;

const SOURCE: &str =
    include_str!("../../../../examples/projects/domains/native_mixed_canvas_session_demo/main.ns");
fn registration() -> yir_core::native_resource_session::ResourceSessionRegistration {
    use yir_core::native_resource_session::*;
    ResourceSessionRegistration {
        contract: MIXED_CONTRACT.to_owned(),
        id: "canvas".to_owned(),
        callbacks: ["canvas_open", "canvas_event", "canvas_close"].map(str::to_owned),
        min_words: 1,
        max_words: 4096,
        state_type: Some("Canvas".to_owned()),
        result_type: None,
    }
}

fn program() -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_resource_session_source(SOURCE, &registration(), &Default::default())
        .unwrap()
        .emit_llvm()
        .unwrap()
}

#[test]
fn mixed_canvas_record_reaches_native_resource_lowering() {
    let program = program();
    let layout = registration().mixed_layout(&program.yir).unwrap().unwrap();
    assert_eq!(
        layout.scalar_fields,
        ["width", "height", "events", "cursor"]
    );
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    assert!(bridge.llvm_ir.contains("call i64 @nuis_fn_canvas_event"));
}

#[test]
#[cfg(unix)]
fn native_mixed_canvas_publishes_complete_control_and_pixels_and_releases_owners() {
    let program = program();
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration,
        library.mixed_binding(&program.yir, &bridge),
        vec![0; 6],
        3,
    )
    .unwrap();
    assert_eq!(session.scalar_state(), [3, 2, 0, 0]);
    let mut pixels = vec![0; 6];
    for i in 0..80 {
        pixels[i % 6] = i as i64 + 100;
        session.event(i as i64 + 100).unwrap();
        assert_eq!(session.state(), pixels);
        assert_eq!(
            session.scalar_state(),
            [3, 2, i as i64 + 1, (i as i64 + 1) % 6]
        );
        assert_eq!(library.live_blobs(), 0);
    }
    let expected = session.scalar_state().to_vec();
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(session.scalar_state(), expected);
    assert_eq!(session.state(), pixels);
    assert_eq!(session.close(99).unwrap(), None);
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn mixed_bridge_rejects_scalar_extents_and_tags_before_invocation() {
    use yir_core::native_resource_session::MIXED_CAPABILITY_TAG;
    let program = program();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let callback = library.mixed_callbacks(&bridge)[1];
    let mut pixels = [31; 6];
    let scalars = [3, 2, 9, 0];
    let mut output = [73; 6];
    let mut out_scalars = [83; 4];
    for (tag, count, outcount, null) in [
        (0, 4, 4, false),
        (MIXED_CAPABILITY_TAG, 3, 4, false),
        (MIXED_CAPABILITY_TAG, 4, 3, false),
        (MIXED_CAPABILITY_TAG, 4, 4, true),
    ] {
        let ptr = if null {
            std::ptr::null()
        } else {
            scalars.as_ptr()
        };
        assert_eq!(
            unsafe {
                callback(
                    tag,
                    pixels.as_mut_ptr(),
                    6,
                    ptr,
                    count,
                    7,
                    output.as_mut_ptr(),
                    6,
                    out_scalars.as_mut_ptr(),
                    outcount,
                )
            },
            1
        );
        assert_eq!(pixels, [31; 6]);
        assert_eq!(output, [73; 6]);
        assert_eq!(out_scalars, [83; 4]);
    }
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn mixed_failure_never_publishes_either_half_and_close_cannot_erase_failure() {
    let program = program();
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    for (role, after_write) in [
        (0, false),
        (1, false),
        (2, false),
        (0, true),
        (1, true),
        (2, true),
    ] {
        let start = bridge
            .llvm_ir
            .find(&format!("define i32 @{}(", bridge.symbols[role]))
            .unwrap();
        let mut fault = bridge.llvm_ir[..start].to_owned();
        fault.push_str(&if after_write {
            bridge.llvm_ir[start..].replacen("  ret i32 0", "  ret i32 3", 1)
        } else {
            bridge.llvm_ir[start..].replacen(
                "%same = icmp eq i64 %length, %bytes",
                "%same = icmp eq i64 %length, -1",
                1,
            )
        });
        let library = native::Library::build(SOURCE, &program, &fault);
        let opened = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.mixed_binding(&program.yir, &bridge),
            vec![0; 6],
            3,
        );
        if role == 0 {
            assert!(opened.is_err());
            assert_eq!(library.live_blobs(), 0);
            continue;
        }
        let (mut session, _) = opened.unwrap();
        if role == 1 {
            assert!(session.event(99).is_err());
            session.close(0).unwrap();
        } else {
            assert!(session.close(0).is_err());
        }
        assert_eq!(session.state(), [0; 6]);
        assert_eq!(session.scalar_state(), [3, 2, 0, 0]);
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn mixed_aggregate_allocation_failure_releases_its_bytes_and_never_opens() {
    let program = program();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    let fault = bridge.llvm_ir.replace(
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 5)",
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
    );
    assert_ne!(fault, bridge.llvm_ir);
    let library = native::Library::build(SOURCE, &program, &fault);
    assert!(NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.mixed_binding(&program.yir, &bridge),
        vec![0; 6],
        3
    )
    .is_err());
    assert_eq!(library.live_blobs(), 0);
}

#[test]
fn mixed_layout_identity_and_linear_owner_admission_are_not_scalar_grants() {
    use yir_core::native_resource_session::MixedResourceLayout;
    for bad in [
        "C{x:i64}",
        "C{x:i64;p:Bytes;q:Bytes}",
        "C{x:bool;p:Bytes}",
        "C{x:i64;x:i64;p:Bytes}",
        "C{x:X{v:i64};p:Bytes}",
    ] {
        assert!(MixedResourceLayout::parse(bad).is_err(), "{bad}");
    }
    let program = program();
    let registration = registration();
    for function in registration.bind(&program.yir).unwrap() {
        assert!(yir_core::validate_session_function(function).is_err());
    }
    let mut changed = registration.clone();
    changed.state_type = Some("Other".to_owned());
    assert!(changed.bind(&program.yir).is_err());
    let mut changed = program.yir.clone();
    let event = changed
        .functions
        .iter_mut()
        .find(|f| f.name == "canvas_event")
        .unwrap();
    event.parameters[1].name = "height".to_owned();
    event.parameters[2].name = "width".to_owned();
    assert!(registration
        .bind(&changed)
        .unwrap_err()
        .contains("field order drift"));
    for replacement in [
        "Canvas{width:i64;height:i64;events:i64;cursor:i64;pixels:Bytes}",
        "Canvas{height:i64;width:i64;events:i64;cursor:i64;pixels:Bytes}",
    ] {
        let mut changed = program.yir.clone();
        let result = changed
            .functions
            .iter()
            .find(|f| f.name == "canvas_event")
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .node
            .clone();
        let node = changed.nodes.iter_mut().find(|n| n.name == result).unwrap();
        if node.op.args[1] == replacement {
            continue;
        }
        node.op.args[1] = replacement.to_owned();
        assert!(registration.bind(&changed).is_err());
    }
    let mut changed = program.yir.clone();
    let open = changed
        .functions
        .iter_mut()
        .find(|f| f.name == "canvas_open")
        .unwrap();
    let copy = changed
        .nodes
        .iter()
        .find(|n| open.body_nodes.contains(&n.name) && n.op.instruction == "copy_buffer_owned")
        .unwrap()
        .name
        .clone();
    let record = changed
        .nodes
        .iter()
        .find(|n| {
            open.body_nodes.contains(&n.name)
                && n.op.instruction == "struct"
                && n.op.args[0] == "Canvas"
        })
        .unwrap()
        .name
        .clone();
    let returned = open.result.as_ref().unwrap().node.clone();
    open.body_nodes.push("invalid_owner_read".to_owned());
    changed
        .node_lanes
        .insert("invalid_owner_read".to_owned(), "fn:canvas_open".to_owned());
    changed.nodes.push(yir_core::Node {
        name: "invalid_owner_read".to_owned(),
        resource: "cpu0".to_owned(),
        op: yir_core::Operation {
            module: "cpu".to_owned(),
            instruction: "owned_bytes_len".to_owned(),
            args: vec![copy],
        },
    });
    for (from, to) in [
        (record, "invalid_owner_read".to_owned()),
        ("invalid_owner_read".to_owned(), returned),
    ] {
        changed.edges.push(yir_core::Edge {
            from,
            to,
            kind: yir_core::EdgeKind::Dep,
        });
    }
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration).is_err()
    );
}

#[test]
#[cfg(unix)]
fn mixed_transport_is_not_a_canvas_specific_field_or_arity_combination() {
    for (count, resource_slot) in [(1, 0), (4, 2), (16, 16)] {
        let names = (0..count).map(|i| format!("s{i}")).collect::<Vec<_>>();
        let mut fields = names
            .iter()
            .map(|s| format!("{s}: i64"))
            .collect::<Vec<_>>();
        fields.insert(resource_slot, "data: Bytes".to_owned());
        let params = names
            .iter()
            .map(|s| format!(", {s}: i64"))
            .collect::<String>();
        let result = |update: bool, open: bool| {
            let mut fields = names
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    format!(
                        "{s}: {}",
                        if open {
                            format!("value + {i}")
                        } else if update {
                            format!("{s} + value")
                        } else {
                            s.clone()
                        }
                    )
                })
                .collect::<Vec<_>>();
            fields.insert(resource_slot, "data: copy_bytes(buffer)".to_owned());
            format!("State {{ {} }}", fields.join(","))
        };
        let source = format!("mod cpu Main {{ struct State {{ {} }} fn start(buffer: ref Buffer, value: i64) -> State {{ return {}; }} fn step(buffer: ref Buffer{params}, value: i64) -> State {{ return {}; }} fn stop(buffer: ref Buffer{params}, value: i64) -> State {{ return {}; }} fn main() -> i64 {{ return 0; }} }}", fields.join(","), result(false, true), result(true, false), result(false, false));
        let mut registration = registration();
        registration.state_type = Some("State".to_owned());
        registration.callbacks = ["start", "step", "stop"].map(str::to_owned);
        let program = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration,
            &Default::default(),
        )
        .unwrap()
        .emit_llvm()
        .unwrap();
        let bridge =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
                .unwrap();
        let library = native::Library::build(&source, &program, &bridge.llvm_ir);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.mixed_binding(&program.yir, &bridge),
            vec![7, -9],
            i64::MAX - 16,
        )
        .unwrap();
        let expected = (0..count)
            .map(|i| i64::MAX - 16 + i as i64)
            .collect::<Vec<_>>();
        assert_eq!(session.scalar_state(), expected);
        session.event(i64::MIN).unwrap();
        assert_eq!(
            session.scalar_state(),
            expected
                .iter()
                .map(|v| v.wrapping_add(i64::MIN))
                .collect::<Vec<_>>()
        );
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(session.state(), [7, -9]);
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn mixed_descriptor_rejects_identity_and_version_before_callbacks() {
    use std::{
        ffi::CString,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use yir_runtime_host::native_resource_session::{
        nuis_native_mixed_resource_script_main as entry, NativeMixedResourceDescriptorV1,
    };
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    unsafe extern "C" fn marker(
        _: u64,
        _: *mut i64,
        _: u64,
        _: *const i64,
        _: u64,
        _: i64,
        _: *mut i64,
        _: u64,
        _: *mut i64,
        _: u64,
    ) -> i32 {
        CALLS.fetch_add(1, Ordering::SeqCst);
        1
    }
    let mut program = program();
    program.yir.resource_sessions.push(registration());
    let source = nuisc::render::render_yir(&program.yir);
    let declaration = registration().canonical();
    let exports = registration().symbols().join("\n");
    let descriptor = || NativeMixedResourceDescriptorV1 {
        version: 2,
        source: source.as_ptr(),
        source_len: source.len() as u64,
        registration: declaration.as_ptr(),
        registration_len: declaration.len() as u64,
        exports: exports.as_ptr(),
        exports_len: exports.len() as u64,
        open: Some(marker),
        event: Some(marker),
        close: Some(marker),
    };
    let args = [
        "app",
        "--resource-session",
        "canvas",
        "--buffer-words",
        "0,0,0,0,0,0",
        "--open-arg",
        "3",
        "--close-arg",
        "0",
    ]
    .map(|s| CString::new(s).unwrap());
    let pointers = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    let run = |d: &NativeMixedResourceDescriptorV1| unsafe {
        entry(
            source.as_ptr(),
            source.len() as u64,
            d,
            pointers.len() as i32,
            pointers.as_ptr(),
        )
    };
    let mut wrong = descriptor();
    wrong.version = 1;
    assert_eq!(run(&wrong), 1);
    let mut wrong = descriptor();
    wrong.close = None;
    assert_eq!(run(&wrong), 1);
    let bad_exports = exports.replace("open", "event");
    let mut wrong = descriptor();
    wrong.exports = bad_exports.as_ptr();
    wrong.exports_len = bad_exports.len() as u64;
    assert_eq!(run(&wrong), 1);
    let bad_registration = declaration.replace("Canvas", "Other");
    let mut wrong = descriptor();
    wrong.registration = bad_registration.as_ptr();
    wrong.registration_len = bad_registration.len() as u64;
    assert_eq!(run(&wrong), 1);
    let mut changed = program.yir.clone();
    changed.resource_sessions[0].max_words = 4095;
    let changed = nuisc::render::render_yir(&changed);
    let mut wrong = descriptor();
    wrong.source = changed.as_ptr();
    wrong.source_len = changed.len() as u64;
    assert_eq!(run(&wrong), 1);
    assert_eq!(CALLS.load(Ordering::SeqCst), 0);
}
