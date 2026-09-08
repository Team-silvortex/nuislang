use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use yir_core::{
    ExecutionState, InstructionSemantics, Node, Operation, RegisteredExecution,
    RegisteredExecutionStep, RegisteredMod, Resource, ResourceKind, Value, YirFunction,
    YirFunctionResult, YirFunctionRole, YirModule, YirValueOwnership,
};

struct CooperativeMod(Arc<AtomicUsize>);
struct Execution {
    function: String,
    calls: Arc<AtomicUsize>,
    waiting: bool,
}

impl RegisteredExecution for Execution {
    fn resume(
        &mut self,
        _: &mut ExecutionState,
        result: Option<Value>,
    ) -> Result<RegisteredExecutionStep, String> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.waiting {
            return Ok(RegisteredExecutionStep::Complete(
                result.ok_or("missing call result")?,
            ));
        }
        if self.function == "endless" {
            return Ok(RegisteredExecutionStep::Continue);
        }
        self.waiting = true;
        Ok(RegisteredExecutionStep::Call {
            function: self.function.clone(),
            arguments: vec![],
        })
    }
}

impl RegisteredMod for CooperativeMod {
    fn module_name(&self) -> &'static str {
        "cooperative"
    }
    fn describe(&self, _: &Node, _: &Resource) -> Result<InstructionSemantics, String> {
        Ok(InstructionSemantics::effect(vec![]))
    }
    fn begin_execution(
        &self,
        node: &Node,
        _: &Resource,
        _: &ExecutionState,
    ) -> Result<Option<Box<dyn RegisteredExecution>>, String> {
        Ok(Some(Box::new(Execution {
            function: node.op.args[0].clone(),
            calls: self.0.clone(),
            waiting: false,
        })))
    }
    fn execute(&self, _: &Node, _: &Resource, _: &mut ExecutionState) -> Result<Value, String> {
        panic!("a cooperative failure must not fall back to another executor")
    }
}

fn module(callee: &str) -> YirModule {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, op, arg) in [
        ("work", "cooperative.run", callee),
        ("answer", "cpu.const_i64", "42"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(op, vec![arg.to_owned()]).unwrap(),
        });
    }
    for (name, node, role) in [
        ("main", "work", YirFunctionRole::Entry),
        ("helper", "answer", YirFunctionRole::Helper),
    ] {
        module.functions.push(YirFunction {
            name: name.to_owned(),
            domain: "cpu".to_owned(),
            role,
            parameters: vec![],
            result: Some(YirFunctionResult {
                ty: "i64".to_owned(),
                ownership: YirValueOwnership::Value,
                node: node.to_owned(),
            }),
            body_nodes: vec![node.to_owned()],
        });
    }
    module
}

#[test]
fn registered_execution_calls_existing_functions_without_domain_dispatch() {
    let mut module = module("helper");
    module.functions[0].role = YirFunctionRole::Helper;
    let count = Arc::new(AtomicUsize::new(0));
    let mut registry = yir_verify::default_registry();
    registry.register(CooperativeMod(count.clone()));
    let mut session =
        yir_exec::FunctionSession::new_rooted_budgeted(&module, &registry, &["main"], 1).unwrap();
    let result = session.invoke_budgeted("main", vec![], 6).unwrap();
    assert_eq!(result.value, Value::Int(42));
    assert_eq!(count.load(Ordering::Relaxed), 2);
}

#[test]
fn registered_execution_and_nested_calls_share_invocation_fuel() {
    let mut module = module("helper");
    module.functions[0].role = YirFunctionRole::Helper;
    let count = Arc::new(AtomicUsize::new(0));
    let mut registry = yir_verify::default_registry();
    registry.register(CooperativeMod(count.clone()));
    let mut session = yir_exec::FunctionSession::new(&module, &registry).unwrap();
    let error = session.invoke_budgeted("main", vec![], 5).unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(
        count.load(Ordering::Relaxed),
        1,
        "fuel must not reset after the helper"
    );
}

#[test]
fn registered_execution_rejects_missing_functions_without_retry() {
    let module = module("missing");
    let count = Arc::new(AtomicUsize::new(0));
    let mut registry = yir_verify::default_registry();
    registry.register(CooperativeMod(count.clone()));
    let error = yir_exec::execute_module_with_registry(&module, &registry).unwrap_err();
    assert!(error.contains("unknown YIR function `missing`"), "{error}");
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[test]
fn registered_execution_is_bounded_even_without_session_fuel() {
    let module = module("endless");
    let count = Arc::new(AtomicUsize::new(0));
    let mut registry = yir_verify::default_registry();
    registry.register(CooperativeMod(count.clone()));
    let error = yir_exec::execute_module_with_registry(&module, &registry).unwrap_err();
    assert!(error.contains("exceeded 100000 reference steps"), "{error}");
    assert_eq!(count.load(Ordering::Relaxed), 100_000);
}
