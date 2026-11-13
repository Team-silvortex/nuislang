use super::{compile, registration, SOURCE};
use yir_core::native_resource_session::{validate_registrations, ResourceSessionRegistration};

#[test]
fn resource_registration_yir_roundtrip_and_drift_rejection() {
    let mut module = compile(SOURCE).yir;
    module.resource_sessions.push(registration());
    let source = nuisc::render::render_yir(&module);
    let parsed = yir_syntax::parse_explicit_module(&source).unwrap();
    assert_eq!(module, parsed);
    validate_registrations(&parsed).unwrap();
    let canonical = registration().canonical();
    assert_eq!(
        ResourceSessionRegistration::from_fields(&canonical.split_whitespace().collect::<Vec<_>>())
            .unwrap(),
        registration()
    );
    for bad in [
        source.replace("nuis-native-resource-session-v1", "unknown-contract"),
        source.replace(&canonical, &canonical.replace(" 1 16", " 01 16")),
        format!("{source}\nresource-session {canonical}\n"),
    ] {
        assert!(yir_syntax::parse_explicit_module(&bad).is_err());
    }
    let mut bad = parsed.clone();
    bad.resource_sessions[0].callbacks[1] = "missing".to_owned();
    assert!(yir_verify::verify_module(&bad).is_err());
    let mut bad = parsed;
    bad.resource_sessions.push(registration());
    assert!(yir_verify::verify_module(&bad).is_err());
}

#[test]
fn resource_script_admits_complete_bounded_input_before_callbacks() {
    use yir_runtime_host::native_resource_session::ResourceScript;
    let mut reg = registration();
    reg.min_words = 0;
    reg.max_words = 4096;
    let base = [
        "--resource-session",
        "bytes",
        "--buffer-words",
        "",
        "--open-arg",
        "0",
        "--close-arg",
        "-1",
    ];
    assert!(ResourceScript::parse(&base, &reg)
        .unwrap()
        .initial
        .is_empty());
    let maximal = vec![i64::MIN.to_string(); 4096].join(",");
    let mut args = base.to_vec();
    args[3] = &maximal;
    assert_eq!(
        ResourceScript::parse(&args, &reg).unwrap().initial.len(),
        4096
    );
    for _ in 0..256 {
        args.extend(["--event-arg", "1"]);
    }
    assert_eq!(
        ResourceScript::parse(&args, &reg).unwrap().events.len(),
        256
    );
    args.extend(["--event-arg", "1"]);
    assert!(ResourceScript::parse(&args, &reg).is_err());
    for value in ["+1", "01", "-0", "9223372036854775808", "bad"] {
        let mut bad = base.to_vec();
        bad[7] = value;
        assert!(ResourceScript::parse(&bad, &reg).is_err());
    }
    let too_many = format!("{maximal},0");
    let mut bad = base.to_vec();
    bad[3] = &too_many;
    assert!(ResourceScript::parse(&bad, &reg).is_err());
    for extra in [
        ["--close-arg", "0"],
        ["--buffer-words", ""],
        ["--resource-session", "bytes"],
        ["--open-args", "0"],
    ] {
        let mut bad = base.to_vec();
        bad.extend(extra);
        assert!(ResourceScript::parse(&bad, &reg).is_err());
    }
}

#[cfg(unix)]
#[test]
fn resource_static_descriptor_checks_graph_registration_and_exports() {
    use std::ffi::CString;
    use yir_runtime_host::native_resource_session::{
        nuis_native_resource_script_main as entry, NativeResourceDescriptorV1,
    };
    let mut program = compile(SOURCE);
    program.yir.resource_sessions.push(registration());
    let source = nuisc::render::render_yir(&program.yir);
    let declaration = registration().canonical();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    let library = super::native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let callbacks = library.callbacks(&bridge);
    let exports = bridge.symbols.join("\n");
    let descriptor = || NativeResourceDescriptorV1 {
        version: 1,
        source: source.as_ptr(),
        source_len: source.len() as u64,
        registration: declaration.as_ptr(),
        registration_len: declaration.len() as u64,
        exports: exports.as_ptr(),
        exports_len: exports.len() as u64,
        open: Some(callbacks[0]),
        event: Some(callbacks[1]),
        close: Some(callbacks[2]),
    };
    let args = [
        "app",
        "--resource-session",
        "bytes",
        "--buffer-words",
        "0,-17",
        "--open-arg",
        "10",
        "--event-arg",
        "3",
        "--close-arg",
        "5",
    ]
    .map(|s| CString::new(s).unwrap());
    let pointers = args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    let run = |d: &NativeResourceDescriptorV1| unsafe {
        entry(
            source.as_ptr(),
            source.len() as u64,
            d,
            pointers.len() as i32,
            pointers.as_ptr(),
        )
    };
    assert_eq!(run(&descriptor()), 0);
    assert_eq!(library.live_blobs(), 0);
    // If an identity gate is accidentally bypassed these valid callbacks trap;
    // this gives a direct pre-callback witness, not merely missing host logs.
    let trapping = SOURCE.replace(
        "store_at(buffer, 0, delta);",
        "store_at(buffer, 999999, delta);",
    );
    let trap_program = compile(&trapping);
    let trap_bridge = yir_lower_llvm::native_resource_session::emit_registered(
        &trap_program.yir,
        &registration(),
    )
    .unwrap();
    let trap_library =
        super::native::Library::build(&trapping, &trap_program, &trap_bridge.llvm_ir);
    let trap_callbacks = trap_library.callbacks(&trap_bridge);
    let invalid = || {
        let mut d = descriptor();
        d.open = Some(trap_callbacks[0]);
        d
    };
    let mut bad = invalid();
    bad.version = 2;
    assert_eq!(run(&bad), 1);
    let mut bad = invalid();
    bad.close = None;
    assert_eq!(run(&bad), 1);
    let wrong_exports = exports.replace("open", "event");
    let mut bad = invalid();
    bad.exports = wrong_exports.as_ptr();
    bad.exports_len = wrong_exports.len() as u64;
    assert_eq!(run(&bad), 1);
    let wrong_registration = declaration.replace(" 1 16", " 2 16");
    let mut bad = invalid();
    bad.registration = wrong_registration.as_ptr();
    bad.registration_len = wrong_registration.len() as u64;
    assert_eq!(run(&bad), 1);
    let mut changed = program.yir.clone();
    changed.resource_sessions[0].max_words = 15;
    let wrong_graph = nuisc::render::render_yir(&changed);
    let mut bad = invalid();
    bad.source = wrong_graph.as_ptr();
    bad.source_len = wrong_graph.len() as u64;
    assert_eq!(run(&bad), 1);
    let mut bad_args = args.to_vec();
    bad_args[10] = CString::new("bad").unwrap();
    let pointers = bad_args.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
    assert_eq!(
        unsafe {
            entry(
                source.as_ptr(),
                source.len() as u64,
                &invalid(),
                pointers.len() as i32,
                pointers.as_ptr(),
            )
        },
        1
    );
    assert_eq!(library.live_blobs(), 0);
    assert_eq!(trap_library.live_blobs(), 0);
}
