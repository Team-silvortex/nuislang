use super::*;
use crate::{ApplicationSession, ApplicationSessionPhase};
use std::cell::RefCell;

const SOURCE: &str = r#"yir 0.1
application-session counter nuis-yir-application-session-v1 open update close state
resource cpu0 cpu.arm64
function open cpu helper
function-param open seed i64 value seed
function-param open ready bool value ready
function-result open Counter owned opened
function-node open seed
function-node open ready
function-node open opened
function update cpu helper
function-param update state.count i64 value count
function-param update state.ready bool value current_ready
function-param update delta i64 value delta
function-result update Counter owned updated
function-node update count
function-node update current_ready
function-node update delta
function-node update next
function-node update updated
function close cpu helper
function-param close state.count i64 value final_count
function-param close state.ready bool value final_ready
function-param close reason i64 value reason
function-result close Counter owned closed
function-node close final_count
function-node close final_ready
function-node close reason
function-node close final
function-node close closed
function main cpu entry
function-node main main_print
cpu.param_i64 seed cpu0 0
cpu.param_bool ready cpu0 1
cpu.struct opened cpu0 Counter count=seed ready=ready
cpu.param_i64 count cpu0 0
cpu.param_bool current_ready cpu0 1
cpu.param_i64 delta cpu0 2
cpu.add next cpu0 count delta
cpu.struct updated cpu0 Counter count=next ready=current_ready
cpu.param_i64 final_count cpu0 0
cpu.param_bool final_ready cpu0 1
cpu.param_i64 reason cpu0 2
cpu.add final cpu0 final_count reason
cpu.struct closed cpu0 Counter count=final ready=final_ready
cpu.const_i64 sentinel cpu0 999
cpu.print main_print cpu0 sentinel
edge dep seed opened
edge dep ready opened
edge dep count next
edge dep delta next
edge dep next updated
edge dep current_ready updated
edge dep final_count final
edge dep reason final
edge dep final closed
edge dep final_ready closed
edge dep sentinel main_print
"#;

#[derive(Default)]
struct Probe {
    calls: [usize; 3],
    statuses: [i32; 3],
    bad_event: bool,
}
thread_local! { static PROBE: RefCell<Probe> = RefCell::default(); }

unsafe fn callback(role: usize, input: *const u64, argc: u64, output: *mut u64, outc: u64) -> i32 {
    let expected = if role == 0 { 2 } else { 3 };
    if argc != expected || outc != 2 || input.is_null() || output.is_null() {
        return 1;
    }
    let input = unsafe { std::slice::from_raw_parts(input, argc as usize) };
    let output = unsafe { std::slice::from_raw_parts_mut(output, 2) };
    PROBE.with_borrow_mut(|p| {
        p.calls[role] += 1;
        output[0] = input[0].wrapping_add(if role == 0 { 0 } else { input[2] });
        output[1] = if role == 1 && p.bad_event {
            2
        } else {
            input[1]
        };
        p.statuses[role]
    })
}
unsafe extern "C" fn open(a: *const u64, n: u64, o: *mut u64, c: u64) -> i32 {
    unsafe { callback(0, a, n, o, c) }
}
unsafe extern "C" fn event(a: *const u64, n: u64, o: *mut u64, c: u64) -> i32 {
    unsafe { callback(1, a, n, o, c) }
}
unsafe extern "C" fn close(a: *const u64, n: u64, o: *mut u64, c: u64) -> i32 {
    unsafe { callback(2, a, n, o, c) }
}

fn module() -> YirModule {
    PROBE.with_borrow_mut(|p| *p = Probe::default());
    yir_syntax::parse_explicit_module(SOURCE).unwrap()
}
fn binding(module: &YirModule) -> NativeSessionBindings<'_> {
    // Test exports implement the scalar contract and inject recoverable boundary
    // failures. This is host-policy coverage, not native lowering evidence.
    unsafe {
        NativeSessionBindings::from_static(
            module,
            "counter",
            "Counter{count:i64;ready:bool}",
            [open, event, close],
        )
        .unwrap()
    }
}
fn opened(module: &YirModule) -> ApplicationSession<'_> {
    let (session, trace) = ApplicationSession::open_native_registered(
        module,
        "counter",
        binding(module),
        vec![Value::Int(10), Value::Bool(true)],
    )
    .unwrap();
    assert_eq!(trace.events, ["native-static-session open"]);
    assert!(trace.lane_steps.is_empty() && trace.presented_frames.is_empty());
    session
}
fn count(session: &ApplicationSession<'_>) -> i64 {
    let Value::Struct(value) = session.state() else {
        panic!("state")
    };
    let Value::Int(count) = value.fields[0].1 else {
        panic!("count")
    };
    count
}

#[test]
fn native_host_admits_arguments_and_rejects_reference_fuel_before_entry() {
    let module = module();
    let mut session = opened(&module);
    assert!(session.event(vec![Value::Bool(true)]).is_err());
    assert!(session
        .event_budgeted(vec![Value::Int(2)], 1)
        .unwrap_err()
        .contains("cannot preempt"));
    assert!(session.close_budgeted(vec![Value::Int(2)], 1).is_err());
    assert!(session.close(vec![]).is_err());
    assert_eq!(session.phase(), ApplicationSessionPhase::Open);
    PROBE.with_borrow(|p| assert_eq!(p.calls, [1, 0, 0]));
    session.event(vec![Value::Int(3)]).unwrap();
    assert_eq!(count(&session), 13);
    session.close(vec![Value::Int(5)]).unwrap();
    assert_eq!(count(&session), 18);
    session.completion_status().unwrap();
    assert!(session.close(vec![]).unwrap().is_none());
    assert!(session.event(vec![Value::Int(1)]).is_err());
    PROBE.with_borrow(|p| assert_eq!(p.calls, [1, 1, 1]));
}

#[test]
fn native_event_failure_retains_last_state_and_cannot_turn_cleanup_into_success() {
    for status in [1, 2, 77] {
        let module = module();
        let mut session = opened(&module);
        session.event(vec![Value::Int(3)]).unwrap();
        PROBE.with_borrow_mut(|p| p.statuses[1] = status);
        assert!(session
            .event(vec![Value::Int(100)])
            .unwrap_err()
            .contains(&format!("status {status}")));
        assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
        assert_eq!(count(&session), 13);
        assert!(session.event(vec![Value::Int(1)]).is_err());
        session.close(vec![Value::Int(5)]).unwrap();
        assert_eq!(count(&session), 18);
        assert!(session.completion_status().is_err());
        assert!(session.close(vec![]).unwrap().is_none());
        PROBE.with_borrow(|p| assert_eq!(p.calls, [1, 2, 1]));
    }
}

#[test]
fn malformed_native_output_and_failed_close_never_publish_partial_state_or_retry() {
    let module = module();
    let mut session = opened(&module);
    PROBE.with_borrow_mut(|p| p.bad_event = true);
    assert!(session
        .event(vec![Value::Int(100)])
        .unwrap_err()
        .contains("noncanonical"));
    assert_eq!(count(&session), 10);
    PROBE.with_borrow_mut(|p| p.statuses[2] = 2);
    let error = session.close(vec![Value::Int(5)]).unwrap_err();
    assert_eq!(session.close(vec![Value::Int(99)]).unwrap_err(), error);
    assert_eq!(session.phase(), ApplicationSessionPhase::Closed);
    assert_eq!(count(&session), 10);
    assert!(session
        .completion_status()
        .unwrap_err()
        .contains("cleanup failed"));
    PROBE.with_borrow(|p| assert_eq!(p.calls, [1, 1, 1]));
}

#[test]
fn native_binding_rejects_changed_graph_registration_layout_and_open_before_entry() {
    let module = module();
    let mut changed = module.clone();
    changed
        .nodes
        .iter_mut()
        .find(|n| n.name == "sentinel")
        .unwrap()
        .op
        .args[0] = "1000".to_owned();
    for (requested, id, args) in [
        (&changed, "counter", vec![Value::Int(10), Value::Bool(true)]),
        (&module, "wrong", vec![Value::Int(10), Value::Bool(true)]),
        (&module, "counter", vec![Value::Int(10)]),
    ] {
        assert!(
            ApplicationSession::open_native_registered(requested, id, binding(&module), args)
                .is_err()
        );
    }
    for layout in [
        "Counter{ready:bool;count:i64}",
        "Other{count:i64;ready:bool}",
        "Counter{count:i64;ready:Bytes}",
        "Counter{count:i64;count:bool}",
    ] {
        assert!(unsafe {
            NativeSessionBindings::from_static(&module, "counter", layout, [open, event, close])
        }
        .is_err());
    }
    PROBE.with_borrow(|p| assert_eq!(p.calls, [0, 0, 0]));
}

#[test]
fn native_open_failure_and_drop_do_not_invent_cleanup_or_replay() {
    let module = module();
    PROBE.with_borrow_mut(|p| p.statuses[0] = 2);
    assert!(ApplicationSession::open_native_registered(
        &module,
        "counter",
        binding(&module),
        vec![Value::Int(10), Value::Bool(true)]
    )
    .is_err());
    PROBE.with_borrow(|p| assert_eq!(p.calls, [1, 0, 0]));
    PROBE.with_borrow_mut(|p| p.statuses[0] = 0);
    drop(opened(&module));
    PROBE.with_borrow(|p| assert_eq!(p.calls, [2, 0, 0]));
}

#[test]
fn native_host_failure_still_requires_one_explicit_cleanup_and_remains_failed() {
    let module = module();
    let mut session = opened(&module);
    session.record_host_failure(yir_core::ApplicationFailureKind::Callback);
    assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
    session.close(vec![Value::Int(5)]).unwrap();
    assert_eq!(count(&session), 15);
    assert!(session.completion_status().is_err());
    PROBE.with_borrow(|p| assert_eq!(p.calls, [1, 0, 1]));
}

#[test]
fn native_static_descriptor_rejects_drift_and_bad_scripts_before_any_callback() {
    use std::ffi::CString;
    let _ = module();
    let descriptor = || NativeSessionDescriptorV1 {
        version: 1,
        source: SOURCE.as_ptr(),
        source_len: SOURCE.len() as u64,
        id: b"counter".as_ptr(),
        id_len: 7,
        layout: b"Counter{count:i64;ready:bool}".as_ptr(),
        layout_len: b"Counter{count:i64;ready:bool}".len() as u64,
        open: Some(open),
        event: Some(event),
        close: Some(close),
    };
    let run = |descriptor: &NativeSessionDescriptorV1, source: &str, options: &[&str]| {
        let args = std::iter::once("native-session")
            .chain(options.iter().copied())
            .map(|arg| CString::new(arg).unwrap())
            .collect::<Vec<_>>();
        let pointers = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
        unsafe {
            nuis_native_application_script_main(
                source.as_ptr(),
                source.len() as u64,
                descriptor,
                pointers.len() as i32,
                pointers.as_ptr(),
            )
        }
    };
    let args = [
        "--application-session",
        "counter",
        "--open-args",
        "10,true",
        "--close-args",
        "5",
    ];
    let mut descriptor = descriptor();
    assert_eq!(run(&descriptor, &SOURCE.replace("999", "998"), &args), 1);
    descriptor.version = 2;
    assert_eq!(run(&descriptor, SOURCE, &args), 1);
    descriptor.version = 1;
    descriptor.event = None;
    assert_eq!(run(&descriptor, SOURCE, &args), 1);
    descriptor.event = Some(event);
    descriptor.source_len = u64::MAX;
    assert_eq!(run(&descriptor, SOURCE, &args), 1);
    descriptor.source_len = SOURCE.len() as u64;
    for bad in [
        vec![
            "--application-session",
            "counter",
            "--open-args",
            "10,true",
            "--close-args",
            "true",
        ],
        vec![
            "--application-session",
            "counter",
            "--open-args",
            "10,1",
            "--close-args",
            "5",
        ],
        vec![
            "--application-session",
            "wrong",
            "--open-args",
            "10,true",
            "--close-args",
            "5",
        ],
    ] {
        assert_eq!(run(&descriptor, SOURCE, &bad), 1);
    }
    PROBE.with_borrow(|p| assert_eq!(p.calls, [0, 0, 0]));
    assert_eq!(run(&descriptor, SOURCE, &args), 0);
    PROBE.with_borrow(|p| assert_eq!(p.calls, [1, 0, 1]));
}
