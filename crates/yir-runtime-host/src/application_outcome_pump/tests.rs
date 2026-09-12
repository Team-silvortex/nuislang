use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use yir_core::{
    ApplicationFailureKind, ApplicationOutcome, ExecutionState, InstructionSemantics, Node,
    RegisteredMod, Resource,
};

use super::*;

const SOURCE: &str = r#"yir 0.1
resource cpu0 cpu.arm64
application-session parent nuis-yir-application-session-v1 open update close state
function open cpu helper
function-result open State owned opened
function-node open opened
function update cpu helper
function-param update state.count i64 value count
function-param update status i64 value status
function-param update cleanup i64 value cleanup
function-param update failure i64 value failure
function-result update State owned updated
function-node update count
function-node update status
function-node update cleanup
function-node update failure
function-node update next
function-node update event_print
function-node update updated
function close cpu helper
function-param close state.count i64 value final_count
function-result close State owned closed
function-node close final_count
function-node close close_print
function-node close closed
cpu.const_i64 seed cpu0 10
cpu.print global_print cpu0 seed
cpu.struct opened cpu0 State count=seed
cpu.param_i64 count cpu0 0
cpu.param_i64 status cpu0 1
cpu.param_i64 cleanup cpu0 2
cpu.param_i64 failure cpu0 3
cpu.add next cpu0 count failure
cpu.print event_print cpu0 next
cpu.struct updated cpu0 State count=next
cpu.param_i64 final_count cpu0 0
cpu.print close_print cpu0 final_count
cpu.struct closed cpu0 State count=final_count
edge dep seed global_print
edge dep seed opened
edge dep count next
edge dep failure next
edge dep next event_print
edge dep event_print updated
edge dep next updated
edge dep final_count close_print
edge dep close_print closed
edge dep final_count closed
"#;

struct CountingCpu {
    base: ModRegistry,
    calls: Arc<AtomicUsize>,
}
impl RegisteredMod for CountingCpu {
    fn module_name(&self) -> &'static str {
        "cpu"
    }
    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        self.base.lookup("cpu").unwrap().describe(node, resource)
    }
    fn function_exit(
        &self,
        node: &Node,
        resource: &Resource,
        state: &ExecutionState,
    ) -> Result<Option<Value>, String> {
        self.base
            .lookup("cpu")
            .unwrap()
            .function_exit(node, resource, state)
    }
    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        if node.op.instruction == "print" {
            self.calls.fetch_add(1, Ordering::Relaxed);
        }
        self.base
            .lookup("cpu")
            .unwrap()
            .execute(node, resource, state)
    }
}

fn spawn(source: &str, limit: usize) -> (ApplicationOutcomePump, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = yir_verify::default_registry();
    registry.register(CountingCpu {
        base: yir_verify::default_registry(),
        calls: calls.clone(),
    });
    (
        ApplicationOutcomePump::spawn(source.to_owned(), "parent".to_owned(), registry, limit)
            .unwrap(),
        calls,
    )
}

fn reply(pump: &mut ApplicationOutcomePump) -> ApplicationOutcomePumpReply {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(reply) = pump.poll().unwrap() {
            return reply;
        }
        assert!(Instant::now() < deadline, "parent worker did not reply");
        thread::sleep(Duration::from_millis(1));
    }
}

fn late_failure() -> ApplicationOutcomeDelivery {
    ApplicationOutcomeDelivery::new(ApplicationOutcome::failed(
        true,
        ApplicationFailureKind::ProviderFinalization,
    ))
}

fn count(reply: &ApplicationOutcomePumpReply) -> i64 {
    let Value::Struct(state) = reply.state.as_ref().unwrap() else {
        panic!("scalar aggregate");
    };
    let Value::Int(count) = state.fields[0].1 else {
        panic!("scalar count");
    };
    count
}

#[test]
fn existing_parent_receives_once_and_closes_without_reinitialization() {
    let (mut pump, calls) = spawn(SOURCE, 100);
    let opened = reply(&mut pump);
    opened.result.as_ref().unwrap();
    assert_eq!(opened.phase, ApplicationOutcomePumpPhase::Open);
    assert_eq!(count(&opened), 10);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(pump.poll().unwrap().is_none());
    pump.finish_with_outcome(late_failure()).unwrap();
    assert!(pump.finish_with_outcome(late_failure()).is_err());
    let closed = reply(&mut pump);
    closed.result.as_ref().unwrap();
    assert_eq!(closed.phase, ApplicationOutcomePumpPhase::Closed);
    assert!(closed.delivered && closed.cleanup_completed);
    assert_eq!(count(&closed), 21);
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    assert!(pump.finish_with_outcome(late_failure()).is_err());
    assert!(pump.poll().unwrap().is_none());
}

#[test]
fn failed_parent_event_has_independent_cleanup_fuel_but_never_success() {
    let (mut pump, calls) = spawn(SOURCE, 3);
    reply(&mut pump).result.unwrap();
    pump.finish_with_outcome(late_failure()).unwrap();
    let failed = reply(&mut pump);
    assert!(failed
        .result
        .as_ref()
        .unwrap_err()
        .contains("step budget exhausted"));
    assert_eq!(failed.phase, ApplicationOutcomePumpPhase::Stopped);
    assert!(!failed.delivered && failed.cleanup_completed);
    assert_eq!(count(&failed), 10);
    assert_eq!(
        calls.load(Ordering::Relaxed),
        3,
        "admitted effects are not rolled back"
    );
}

#[test]
fn initialization_and_open_are_budgeted_and_never_retried() {
    for (source, expected_calls) in [
        (SOURCE.to_owned(), 0),
        (
            SOURCE
                .replace("cpu.print global_print cpu0 seed\n", "")
                .replace("edge dep seed global_print\n", ""),
            0,
        ),
    ] {
        let (mut pump, calls) = spawn(&source, 1);
        let failed = reply(&mut pump);
        assert!(failed.result.unwrap_err().contains("step budget exhausted"));
        assert!(failed.state.is_none());
        assert!(!failed.delivered && !failed.cleanup_completed);
        assert_eq!(calls.load(Ordering::Relaxed), expected_calls);
        assert!(pump.finish_with_outcome(late_failure()).is_err());
    }
}

#[test]
fn signature_rejection_precedes_effects_and_drop_does_not_cleanup() {
    let bad = SOURCE.replace(
        "function-param update failure i64",
        "function-param update failure i32",
    );
    let (mut pump, calls) = spawn(&bad, 100);
    assert!(reply(&mut pump)
        .result
        .unwrap_err()
        .contains("expects value-owned"));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    let calls = {
        let (mut pump, calls) = spawn(SOURCE, 100);
        reply(&mut pump).result.unwrap();
        calls
    };
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[test]
fn packaged_profile_rejects_unadmitted_execution_without_reference_fallback() {
    let source =
        format!("{SOURCE}resource link data.fabric\ndata.handle_table handles link host=cpu0\n");
    let module = yir_syntax::parse_module(&source).unwrap();
    let registry = host_registry::cpu_parent_registry(&module).unwrap();
    let node = module.nodes.last().unwrap();
    let resource = module.resources.last().unwrap();
    let mut state = ExecutionState::default();
    let error = registry
        .lookup("data")
        .unwrap()
        .execute(node, resource, &mut state)
        .unwrap_err();
    assert!(error.contains("CPU profile cannot execute `data`"));
    // The same global is harmless when unrelated, but cannot be pruned away
    // once an explicit ordering edge makes it part of parent initialization.
    ApplicationSession::open_registered_rooted_budgeted(&module, &registry, "parent", vec![], 100)
        .unwrap();
    let linked = yir_syntax::parse_module(&(source + "edge dep handles opened\n")).unwrap();
    let result = ApplicationSession::open_registered_rooted_budgeted(
        &linked,
        &registry,
        "parent",
        vec![],
        100,
    );
    let Err(error) = result else {
        panic!("unadmitted dependency must reject");
    };
    assert!(error.contains("CPU profile cannot execute `data`"));
}

#[test]
fn exhausted_close_keeps_delivered_state_and_cannot_report_cleanup_success() {
    let source = SOURCE
        .replace("function-node close closed", "function-node close closed\nfunction-node close extra1\nfunction-node close extra2")
        .replace("cpu.struct closed", "cpu.add extra1 cpu0 final_count seed\ncpu.add extra2 cpu0 extra1 seed\ncpu.struct closed")
        .replace("edge dep close_print closed", "edge dep close_print extra1\nedge dep final_count extra1\nedge dep seed extra1\nedge dep extra1 extra2\nedge dep seed extra2\nedge dep extra2 closed");
    let (mut pump, _) = spawn(&source, 4);
    reply(&mut pump).result.unwrap();
    pump.finish_with_outcome(late_failure()).unwrap();
    let failed = reply(&mut pump);
    assert!(failed
        .result
        .as_ref()
        .unwrap_err()
        .contains("step budget exhausted"));
    assert!(failed.delivered && !failed.cleanup_completed);
    assert_eq!(count(&failed), 21);
    assert_eq!(failed.phase, ApplicationOutcomePumpPhase::Stopped);
}

#[test]
fn ffi_delivery_is_one_attempt_and_keeps_child_failure_readable() {
    use crate::{ApplicationProviderSource, ApplicationPumpPhase, WindowSession};
    use std::{path::Path, ptr};
    let mut parent = ptr::null_mut();
    unsafe {
        assert_eq!(
            nuis_outcome_parent_open(
                SOURCE.as_ptr(),
                SOURCE.len(),
                c"parent".as_ptr(),
                &mut parent
            ),
            0
        );
        reply(&mut *parent).result.unwrap();
        assert_eq!(
            nuis_outcome_parent_open(
                SOURCE.as_ptr(),
                SOURCE.len(),
                c"parent".as_ptr(),
                &mut parent
            ),
            -1
        );
        let mut child = WindowSession::spawn(
            "yir 0.1\n".to_owned(),
            ApplicationProviderSource::Replay(Path::new("unused-parent-test")),
            "absent".to_owned(),
            1,
            1,
        )
        .unwrap();
        assert_eq!(nuis_outcome_parent_finish(parent, &mut child), -1);
        let deadline = Instant::now() + Duration::from_secs(5);
        while child.phase() != ApplicationPumpPhase::Stopped {
            let _ = child.poll();
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(nuis_outcome_parent_finish(parent, &mut child), 0);
        assert_eq!(nuis_outcome_parent_finish(parent, &mut child), -1);
        let mut phase = 0;
        let mut delivered = 0;
        let mut cleanup = 0;
        loop {
            let status = nuis_outcome_parent_poll(parent, &mut phase, &mut delivered, &mut cleanup);
            if status != 0 {
                assert_eq!(status, 1);
                break;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!((phase, delivered, cleanup), (3, 1, 1));
        assert_eq!(child.outcome().unwrap().codes(), [2, 0, 8]);
        assert!(child.take_outcome_delivery().is_none());
        nuis_outcome_parent_free(&mut parent);
        nuis_outcome_parent_free(&mut parent);
        assert!(parent.is_null());
    }
}

#[test]
fn ffi_rejects_invalid_handles_and_utf8_without_opening_a_parent() {
    use std::ptr;
    let mut parent = ptr::null_mut();
    unsafe {
        assert_eq!(
            nuis_outcome_parent_open(ptr::null(), 0, c"parent".as_ptr(), &mut parent),
            -1
        );
        assert_eq!(
            nuis_outcome_parent_open([0xff].as_ptr(), 1, c"parent".as_ptr(), &mut parent),
            -1
        );
        assert_eq!(nuis_outcome_parent_finish(parent, ptr::null_mut()), -1);
        assert_eq!(
            nuis_outcome_parent_poll(parent, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()),
            -1
        );
        nuis_outcome_parent_free(&mut parent);
        assert!(parent.is_null());
    }
}

#[test]
fn rooted_initialization_preserves_ordered_effects_but_not_unrelated_globals() {
    let source = format!("{SOURCE}cpu.print unrelated cpu0 seed\nedge dep seed unrelated\nedge dep global_print opened\n");
    let module = yir_syntax::parse_module(&source).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = yir_verify::default_registry();
    registry.register(CountingCpu {
        base: yir_verify::default_registry(),
        calls: calls.clone(),
    });
    let (mut parent, _) = ApplicationSession::open_registered_rooted_budgeted(
        &module,
        &registry,
        "parent",
        vec![],
        100,
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    late_failure().deliver(&mut parent, 100).unwrap();
    parent.close_budgeted(vec![], 100).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 3);
}

#[test]
fn rooted_initialization_follows_helper_globals_and_rejects_unrelated_functions() {
    let source = SOURCE.replace(
        "cpu.add next cpu0 count failure",
        "cpu.call_i64 next cpu0 adjust count",
    ) + r#"
function adjust cpu helper
function-param adjust value i64 value adjusted_input
function-result adjust i64 value adjusted
function-node adjust adjusted_input
function-node adjust adjusted
cpu.const_i64 adjustment cpu0 11
cpu.param_i64 adjusted_input cpu0 0
cpu.add adjusted cpu0 adjusted_input adjustment
edge dep adjusted_input adjusted
edge dep adjustment adjusted
"#;
    let module = yir_syntax::parse_module(&source).unwrap();
    let registry = yir_verify::default_registry();
    let mut execution =
        FunctionSession::new_rooted_budgeted(&module, &registry, &["update"], 100).unwrap();
    assert!(execution
        .invoke("open", vec![])
        .unwrap_err()
        .contains("outside the admitted session roots"));
    let updated = execution
        .invoke(
            "update",
            vec![Value::Int(10), Value::Int(2), Value::Int(1), Value::Int(11)],
        )
        .unwrap();
    let Value::Struct(state) = updated.value else {
        panic!("scalar aggregate");
    };
    assert_eq!(state.fields[0].1, Value::Int(21));
    assert_eq!(
        updated
            .trace
            .events
            .iter()
            .filter(|event| event.contains("effect cpu.print"))
            .count(),
        1,
        "only the event print is admitted"
    );
}
