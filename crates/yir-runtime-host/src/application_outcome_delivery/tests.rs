use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use yir_core::{
    ApplicationFailureKind, ExecutionState, InstructionSemantics, ModRegistry, Node, RegisteredMod,
    Resource, YirModule,
};

use super::*;
use crate::{
    ApplicationProviderSource, ApplicationSessionEntries, ApplicationSessionPhase, WindowSession,
};

struct CountingCpu {
    base: ModRegistry,
    prints: Arc<AtomicUsize>,
}

impl RegisteredMod for CountingCpu {
    fn module_name(&self) -> &'static str {
        "cpu"
    }
    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        self.base.lookup("cpu").unwrap().describe(node, resource)
    }
    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        if node.op.instruction == "print" {
            self.prints.fetch_add(1, Ordering::Relaxed);
        }
        self.base
            .lookup("cpu")
            .unwrap()
            .execute(node, resource, state)
    }
}

fn fixture() -> (YirModule, ModRegistry, Arc<AtomicUsize>) {
    let module = yir_syntax::parse_module(
        r#"
yir 0.1
resource cpu0 cpu.arm64
function open cpu helper
function-param open seed i64 value seed
function-result open State owned opened
function-node open seed
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
function-node close closed
cpu.const_i64 one cpu0 1
cpu.print global_print cpu0 one
cpu.param_i64 seed cpu0 0
cpu.struct opened cpu0 State count=seed
cpu.param_i64 count cpu0 0
cpu.param_i64 status cpu0 1
cpu.param_i64 cleanup cpu0 2
cpu.param_i64 failure cpu0 3
cpu.add next cpu0 count failure
cpu.print event_print cpu0 next
cpu.struct updated cpu0 State count=next
cpu.param_i64 final_count cpu0 0
cpu.struct closed cpu0 State count=final_count
edge dep one global_print
edge dep seed opened
edge dep count next
edge dep failure next
edge dep next event_print
edge dep event_print updated
edge dep next updated
edge dep final_count closed
"#,
    )
    .unwrap();
    let prints = Arc::new(AtomicUsize::new(0));
    let mut registry = yir_verify::default_registry();
    registry.register(CountingCpu {
        base: yir_verify::default_registry(),
        prints: prints.clone(),
    });
    (module, registry, prints)
}

fn entries() -> ApplicationSessionEntries<'static> {
    ApplicationSessionEntries {
        open: "open",
        event: "update",
        close: "close",
        state_parameter: "state",
    }
}

fn count(parent: &ApplicationSession<'_>) -> i64 {
    let Value::Struct(value) = parent.state() else {
        panic!("expected state")
    };
    let Value::Int(count) = value.fields[0].1 else {
        panic!("expected count")
    };
    count
}

#[test]
fn delivery_uses_parent_owned_state_and_effects_without_reinitialization() {
    let (module, registry, prints) = fixture();
    let (mut parent, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    assert_eq!(prints.load(Ordering::Relaxed), 1);
    parent
        .event(vec![Value::Int(2), Value::Int(1), Value::Int(3)])
        .unwrap();
    let child = ApplicationOutcome::failed(true, ApplicationFailureKind::ProviderFinalization);
    let trace = ApplicationOutcomeDelivery::new(child)
        .deliver(&mut parent, 4)
        .unwrap();
    assert_eq!(count(&parent), 24);
    assert_eq!(prints.load(Ordering::Relaxed), 3);
    assert_eq!(trace.lane_steps.values().map(Vec::len).sum::<usize>(), 3);
    assert!(trace.provider_completion_witnesses.is_empty());
    assert_eq!(parent.failure_kind(), ApplicationFailureKind::None);
    parent.close(vec![]).unwrap();
    parent.completion_status().unwrap();
    assert_eq!(child.codes(), [2, 1, 11]);
}

#[test]
fn failed_delivery_keeps_effects_but_not_new_state_and_does_not_authorize_retry() {
    let (module, registry, prints) = fixture();
    let (mut parent, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    let child = ApplicationOutcome::success();
    // Entry + add + print, but no budget remains to return the new state.
    let error = ApplicationOutcomeDelivery::new(child)
        .deliver(&mut parent, 3)
        .unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(prints.load(Ordering::Relaxed), 2);
    assert_eq!(count(&parent), 10);
    assert_eq!(parent.phase(), ApplicationSessionPhase::Faulted);
    assert!(parent
        .event(vec![Value::Int(1), Value::Int(1), Value::Int(0)])
        .is_err());
    assert_eq!(prints.load(Ordering::Relaxed), 2);
    parent.close(vec![]).unwrap();
    assert!(parent.completion_status().is_err());
    assert_eq!(child.codes(), [1, 1, 0]);
}

#[test]
fn signature_and_closed_parent_rejections_have_no_effects() {
    let (mut module, registry, prints) = fixture();
    // The existing event has only two trailing arguments instead of three.
    module.functions[1].parameters.pop();
    module.functions[1]
        .body_nodes
        .retain(|name| name != "failure");
    module
        .nodes
        .iter_mut()
        .find(|node| node.name == "failure")
        .unwrap()
        .op = yir_core::Operation::parse("cpu.const_i64", vec!["0".to_owned()]).unwrap();
    let (mut parent, _) =
        ApplicationSession::open(&module, &registry, entries(), vec![Value::Int(10)]).unwrap();
    let error = ApplicationOutcomeDelivery::new(ApplicationOutcome::success())
        .deliver(&mut parent, 100)
        .unwrap_err();
    assert!(error.contains("scalar arguments"), "{error}");
    assert_eq!(parent.phase(), ApplicationSessionPhase::Open);
    assert_eq!(prints.load(Ordering::Relaxed), 1);
    parent.close(vec![]).unwrap();
    assert!(
        ApplicationOutcomeDelivery::new(ApplicationOutcome::success())
            .deliver(&mut parent, 100)
            .is_err()
    );
    assert_eq!(prints.load(Ordering::Relaxed), 1);
}

#[test]
fn terminal_source_issues_one_attempt_even_if_it_is_abandoned() {
    let mut child = WindowSession::spawn(
        "yir 0.1\n".to_owned(),
        ApplicationProviderSource::Replay(std::path::Path::new("unused-delivery-replay")),
        "absent".to_owned(),
        1,
        1,
    )
    .unwrap();
    assert!(child.take_outcome_delivery().is_none());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while child.outcome().is_none() {
        let _ = child.poll();
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    {
        let delivery = child.take_outcome_delivery().unwrap();
        assert_eq!(delivery.outcome().codes(), [2, 0, 8]);
    }
    assert!(child.take_outcome_delivery().is_none());
    assert_eq!(child.outcome().unwrap().codes(), [2, 0, 8]);
    assert!(child.close().is_err());
}
