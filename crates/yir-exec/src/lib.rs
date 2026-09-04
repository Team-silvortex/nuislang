use std::collections::{BTreeMap, BTreeSet};

use yir_core::{
    ExecutionState, FrameSurface, ModRegistry, Node, ProviderCompletionWitness, Value, YirModule,
};
use yir_verify::default_registry;

mod execution_engine;

#[derive(Debug, Default)]
pub struct ExecutionTrace {
    pub events: Vec<String>,
    pub lane_events: BTreeMap<String, Vec<String>>,
    pub lane_steps: BTreeMap<String, Vec<String>>,
    pub values: BTreeMap<String, Value>,
    pub presented_frames: Vec<FrameSurface>,
    pub provider_completion_witnesses: BTreeMap<String, ProviderCompletionWitness>,
}

pub fn execute_module(module: &YirModule) -> Result<ExecutionTrace, String> {
    let registry = default_registry();
    execute_module_with_registry(module, &registry)
}

pub fn execute_module_with_registry(
    module: &YirModule,
    registry: &ModRegistry,
) -> Result<ExecutionTrace, String> {
    execution_engine::execute_module_with_registry(module, registry)
}

enum LazySelectOutcome {
    Handled(Value),
    UseRegisteredExecutor,
}

fn execute_lazy_select(
    node: &Node,
    state: &mut ExecutionState,
    delayed: &mut BTreeMap<String, String>,
    nodes_by_name: &BTreeMap<String, &Node>,
) -> Result<LazySelectOutcome, String> {
    if node.op.args.len() != 3 {
        return Ok(LazySelectOutcome::UseRegisteredExecutor);
    }
    let cond = match state.expect_value(&node.op.args[0])? {
        Value::Bool(value) => *value,
        Value::Int(value) => *value != 0,
        other => {
            return Err(format!(
                "node `{}` expects bool or i64 select condition, got {}",
                node.name, other
            ))
        }
    };
    let selected = if cond {
        node.op.args[1].as_str()
    } else {
        node.op.args[2].as_str()
    };
    let unselected = if cond {
        node.op.args[2].as_str()
    } else {
        node.op.args[1].as_str()
    };
    if !delayed.contains_key(selected) && !delayed.contains_key(unselected) {
        return Ok(LazySelectOutcome::UseRegisteredExecutor);
    }
    let Some(value) = state.values.get(selected).cloned() else {
        if let Some(error) = delayed.get(selected) {
            return Err(format!(
                "node `{}` selected delayed branch `{selected}`: {error}",
                node.name
            ));
        }
        return Err(format!("missing value for `{selected}`"));
    };
    clear_delayed_dependency_closure(unselected, delayed, nodes_by_name);
    Ok(LazySelectOutcome::Handled(value))
}

fn first_delayed_input<'a>(
    node: &'a Node,
    delayed: &'a BTreeMap<String, String>,
) -> Option<(&'a str, &'a str)> {
    node.op.args.iter().find_map(|arg| {
        let value_name = arg.split_once('=').map_or(arg.as_str(), |(_, value)| value);
        delayed
            .get(value_name)
            .map(|reason| (value_name, reason.as_str()))
    })
}

fn is_delayable_variant_error(node: &Node, error: &str) -> bool {
    node.op.module == "cpu"
        && node.op.instruction == "variant_field"
        && error.contains("expects variant `")
}

fn clear_delayed_dependency_closure(
    root: &str,
    delayed: &mut BTreeMap<String, String>,
    nodes_by_name: &BTreeMap<String, &Node>,
) {
    let mut stack = vec![root.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(name) = stack.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        delayed.remove(&name);
        if let Some(node) = nodes_by_name.get(&name) {
            stack.extend(node.op.args.iter().map(|arg| {
                arg.split_once('=')
                    .map_or_else(|| arg.clone(), |(_, value)| value.to_owned())
            }));
        }
    }
}

fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();

    for node in &module.nodes {
        adjacency.entry(node.name.clone()).or_default();
        indegree.entry(node.name.clone()).or_insert(0);
    }

    for edge in &module.edges {
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        *indegree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
        .collect::<Vec<_>>();
    ready.sort();

    let mut order = Vec::with_capacity(module.nodes.len());

    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target.clone());
                        ready.sort();
                    }
                }
            }
        }
    }

    if order.len() != module.nodes.len() {
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{
        BranchEffectAction, BranchEffectActionCapability, BranchEffectResult, Edge, EdgeKind,
        InstructionSemantics, Operation, ProviderCompletionClockKind,
        ProviderCompletionRegistration, ProviderPhysicalCompletion, RegisteredMod, Resource,
        ResourceKind, YirFunction, YirFunctionParameter, YirFunctionResult, YirFunctionRole,
        YirResultFamily, YirValueOwnership,
    };

    const PROBE_ACTIONS: &[BranchEffectActionCapability] = &[
        BranchEffectActionCapability {
            module: "probe",
            instruction: "left",
            result: BranchEffectResult::I64,
            operand_accesses: &[],
        },
        BranchEffectActionCapability {
            module: "probe",
            instruction: "right",
            result: BranchEffectResult::I64,
            operand_accesses: &[],
        },
    ];

    struct ProbeMod;

    impl RegisteredMod for ProbeMod {
        fn module_name(&self) -> &'static str {
            "probe"
        }

        fn branch_effect_action_capabilities(&self) -> &'static [BranchEffectActionCapability] {
            PROBE_ACTIONS
        }

        fn describe(
            &self,
            node: &Node,
            _resource: &Resource,
        ) -> Result<InstructionSemantics, String> {
            Err(format!("unexpected standalone probe node `{}`", node.name))
        }

        fn execute(
            &self,
            node: &Node,
            _resource: &Resource,
            _state: &mut ExecutionState,
        ) -> Result<Value, String> {
            Err(format!("unexpected standalone probe node `{}`", node.name))
        }

        fn execute_branch_effect_action(
            &self,
            action: &BranchEffectAction<'_>,
            _parent: &Node,
            _resource: &Resource,
            _state: &mut ExecutionState,
        ) -> Result<Value, String> {
            match action.instruction {
                "left" => Ok(Value::Int(41)),
                "right" => Ok(Value::Int(73)),
                other => Err(format!("unknown probe action `{other}`")),
            }
        }
    }

    struct PhysicalProbeMod;

    impl RegisteredMod for PhysicalProbeMod {
        fn module_name(&self) -> &'static str {
            "physical"
        }

        fn provider_completion_registration(
            &self,
            node: &Node,
        ) -> Option<ProviderCompletionRegistration> {
            (node.op.instruction == "submit").then_some(
                ProviderCompletionRegistration::physical_fence_required(
                    YirResultFamily::Shader,
                    "shader.clock.frame.v1",
                ),
            )
        }

        fn describe(
            &self,
            node: &Node,
            _resource: &Resource,
        ) -> Result<InstructionSemantics, String> {
            match node.op.instruction.as_str() {
                "submit" if node.op.args.len() == 1 => Ok(InstructionSemantics::effect(Vec::new())),
                _ => Err(format!("invalid physical probe node `{}`", node.name)),
            }
        }

        fn execute(
            &self,
            node: &Node,
            _resource: &Resource,
            state: &mut ExecutionState,
        ) -> Result<Value, String> {
            let source_clock = node.op.args[0]
                .parse::<i64>()
                .map_err(|_| "physical probe clock is invalid".to_owned())?;
            state.stage_provider_physical_completion(
                node,
                ProviderPhysicalCompletion::new(
                    "shader.clock.frame.v1",
                    "probe.monotonic.v1",
                    "probe.queue-fence.completed",
                    source_clock,
                )?,
            )?;
            Ok(Value::Int(source_clock))
        }
    }

    fn cpu_resource() -> Resource {
        Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.main"),
        }
    }

    fn cpu_node(name: &str, instruction: &str, args: &[&str]) -> Node {
        Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(
                &format!("cpu.{instruction}"),
                args.iter().map(|arg| (*arg).to_owned()).collect(),
            )
            .unwrap(),
        }
    }

    fn dep(from: &str, to: &str) -> Edge {
        Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    #[test]
    fn lazy_select_skips_unselected_variant_field_chain() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("cond", "const_bool", &["false"]),
            cpu_node("payload", "const_i64", &["41"]),
            cpu_node("err", "struct", &["Result.Err", "value=payload"]),
            cpu_node(
                "wrong_payload",
                "variant_field",
                &["err", "Result.Ok", "value"],
            ),
            cpu_node("one", "const_i64", &["1"]),
            cpu_node("bad_sum", "add", &["wrong_payload", "one"]),
            cpu_node("fallback", "const_i64", &["7"]),
            cpu_node("selected", "select", &["cond", "bad_sum", "fallback"]),
        ]);
        module.edges.extend([
            dep("payload", "err"),
            dep("err", "wrong_payload"),
            dep("wrong_payload", "bad_sum"),
            dep("one", "bad_sum"),
            dep("cond", "selected"),
            dep("bad_sum", "selected"),
            dep("fallback", "selected"),
        ]);

        let trace = execute_module(&module).expect("lazy select should skip bad branch");
        assert_eq!(trace.values.get("selected"), Some(&Value::Int(7)));
        assert!(!trace.values.contains_key("wrong_payload"));
        assert!(!trace.values.contains_key("bad_sum"));
    }

    #[test]
    fn injected_registry_executes_cross_mod_branch_action() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("choose", "const_bool", &["false"]),
            cpu_node(
                "selected",
                "branch_effect",
                &[
                    "choose", "i64", "1", "probe", "left", "i64", "0", "1", "probe", "right",
                    "i64", "0",
                ],
            ),
        ]);
        module.edges.push(dep("choose", "selected"));

        let mut registry = default_registry();
        registry.register(ProbeMod);
        let trace = execute_module_with_registry(&module, &registry)
            .expect("registered probe action should execute through composition");
        assert_eq!(trace.values.get("selected"), Some(&Value::Int(73)));
    }

    #[test]
    fn registered_shader_completion_issues_an_implicit_receipt_after_execution() {
        let mut module = YirModule::new("0.1");
        module.resources.push(Resource {
            name: "shader0".to_owned(),
            kind: ResourceKind::parse("shader.reference"),
        });
        let shader_node = |name: &str, instruction: &str, args: &[&str]| Node {
            name: name.to_owned(),
            resource: "shader0".to_owned(),
            op: Operation::parse(
                &format!("shader.{instruction}"),
                args.iter().map(|arg| (*arg).to_owned()).collect(),
            )
            .unwrap(),
        };
        module.nodes.extend([
            shader_node("target", "target", &["rgba8", "8", "8"]),
            shader_node("pipeline", "pipeline", &["flat", "triangle"]),
            shader_node("viewport", "viewport", &["8", "8"]),
            shader_node("pass", "begin_pass", &["target", "pipeline", "viewport"]),
            shader_node("result", "observe", &["pass", "pass_ready"]),
            shader_node("token", "completion_token", &["result"]),
            shader_node("clock", "completion_clock", &["result"]),
            shader_node("root", "completion_root", &["result"]),
        ]);
        module.edges.extend([
            dep("target", "pass"),
            dep("pipeline", "pass"),
            dep("viewport", "pass"),
            dep("pass", "result"),
            dep("result", "token"),
            dep("result", "clock"),
            dep("result", "root"),
        ]);

        let trace = execute_module(&module).expect("registered completion should execute");
        let expected = yir_core::issue_provider_completion_receipt(
            yir_core::YirResultFamily::Shader,
            "shader0",
            "pass",
            "pass_ready",
            1,
        );
        assert_eq!(trace.values.get("token"), Some(&Value::Int(expected.token)));
        assert_eq!(trace.values.get("clock"), Some(&Value::Int(1)));
        assert_eq!(trace.values.get("root"), Some(&Value::Int(expected.root)));
    }

    #[test]
    fn registered_provider_imports_ordered_physical_fences_transactionally() {
        let mut module = YirModule::new("0.1");
        module.resources.push(Resource {
            name: "shader0".to_owned(),
            kind: ResourceKind::parse("shader.reference"),
        });
        module.nodes.extend([
            Node {
                name: "submit0".to_owned(),
                resource: "shader0".to_owned(),
                op: Operation::parse("physical.submit", vec!["100".to_owned()]).unwrap(),
            },
            Node {
                name: "submit1".to_owned(),
                resource: "shader0".to_owned(),
                op: Operation::parse("physical.submit", vec!["101".to_owned()]).unwrap(),
            },
        ]);
        module.edges.push(dep("submit0", "submit1"));
        let mut registry = ModRegistry::new();
        registry.register(PhysicalProbeMod);

        let trace = execute_module_with_registry(&module, &registry)
            .expect("registered physical provider should execute");
        let first = &trace.provider_completion_witnesses["submit0"];
        let second = &trace.provider_completion_witnesses["submit1"];
        assert_eq!(first.completion_clock, 1);
        assert_eq!(second.completion_clock, 2);
        assert_eq!(
            second.clock_kind,
            ProviderCompletionClockKind::PhysicalFence
        );
        assert_eq!(second.physical_source_clock, Some(101));

        module.nodes.push(Node {
            name: "submit2".to_owned(),
            resource: "shader0".to_owned(),
            op: Operation::parse("physical.submit", vec!["100".to_owned()]).unwrap(),
        });
        module.edges.push(dep("submit1", "submit2"));
        let error = execute_module_with_registry(&module, &registry).unwrap_err();
        assert!(error.contains("clock 100 is stale"));
    }

    #[test]
    fn registered_function_call_executes_with_bound_parameters() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("inc_param", "param_i64", &["0"]),
            cpu_node("inc_one", "const_i64", &["1"]),
            cpu_node("inc_value", "add", &["inc_param", "inc_one"]),
            cpu_node("inc_return", "return_i64", &["inc_value"]),
            cpu_node("main_input", "const_i64", &["41"]),
            cpu_node("main_call", "call_i64", &["increment", "main_input"]),
            cpu_node("main_return", "return_i64", &["main_call"]),
        ]);
        module.edges.extend([
            dep("inc_param", "inc_value"),
            dep("inc_one", "inc_value"),
            dep("inc_value", "inc_return"),
            dep("main_input", "main_call"),
            dep("main_call", "main_return"),
        ]);
        module.functions.extend([
            YirFunction {
                name: "increment".to_owned(),
                domain: "cpu".to_owned(),
                role: YirFunctionRole::Helper,
                parameters: vec![YirFunctionParameter {
                    name: "value".to_owned(),
                    ty: "i64".to_owned(),
                    ownership: YirValueOwnership::Value,
                    node: "inc_param".to_owned(),
                }],
                result: Some(YirFunctionResult {
                    ty: "i64".to_owned(),
                    ownership: YirValueOwnership::Value,
                    node: "inc_return".to_owned(),
                }),
                body_nodes: vec![
                    "inc_param".to_owned(),
                    "inc_one".to_owned(),
                    "inc_value".to_owned(),
                    "inc_return".to_owned(),
                ],
            },
            YirFunction {
                name: "main".to_owned(),
                domain: "cpu".to_owned(),
                role: YirFunctionRole::Entry,
                parameters: Vec::new(),
                result: Some(YirFunctionResult {
                    ty: "i64".to_owned(),
                    ownership: YirValueOwnership::Value,
                    node: "main_return".to_owned(),
                }),
                body_nodes: vec![
                    "main_input".to_owned(),
                    "main_call".to_owned(),
                    "main_return".to_owned(),
                ],
            },
        ]);

        let trace = execute_module(&module).expect("registered helper call should execute");
        assert_eq!(trace.values.get("main_call"), Some(&Value::Int(42)));
        assert!(!trace.values.contains_key("inc_param"));
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| event.contains("effect cpu.return_i64"))
                .count(),
            2
        );
    }

    #[test]
    fn scoped_owned_struct_loop_executes_helper_and_carries_result() {
        let layout = "Counter{value:i64}";
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("bump_value", "param_i64", &["0"]),
            cpu_node("bump_current", "param_i64", &["1"]),
            cpu_node("bump_one", "const_i64", &["1"]),
            cpu_node("bump_next", "add", &["bump_value", "bump_one"]),
            cpu_node("bump_struct", "struct", &["Counter", "value=bump_next"]),
            cpu_node("bump_return", "return_owned_struct", &["bump_struct"]),
            cpu_node("initial", "const_i64", &["0"]),
            cpu_node("limit", "const_i64", &["3"]),
            cpu_node("step", "const_i64", &["1"]),
            cpu_node("carry", "const_i64", &["0"]),
            cpu_node(
                "loop",
                "loop_while_i64_effect",
                &[
                    "initial",
                    "limit",
                    "step",
                    "lt",
                    "add",
                    "cpu",
                    "scoped_call_owned_struct_return",
                    "5",
                    "bump",
                    "loop_result",
                    layout,
                    "$owned_struct_carry:0:carry",
                    "$current",
                ],
            ),
            cpu_node("loop_result", "loop_owned_struct_result", &["loop", layout]),
            cpu_node("main_return", "return_i64", &["loop"]),
        ]);
        module.edges.extend([
            dep("bump_value", "bump_next"),
            dep("bump_one", "bump_next"),
            dep("bump_next", "bump_struct"),
            dep("bump_struct", "bump_return"),
            dep("initial", "loop"),
            dep("limit", "loop"),
            dep("step", "loop"),
            dep("carry", "loop"),
            dep("loop", "loop_result"),
            dep("loop", "main_return"),
        ]);
        module.functions.extend([
            YirFunction {
                name: "bump".to_owned(),
                domain: "cpu".to_owned(),
                role: YirFunctionRole::Helper,
                parameters: vec![
                    YirFunctionParameter {
                        name: "value".to_owned(),
                        ty: "i64".to_owned(),
                        ownership: YirValueOwnership::Value,
                        node: "bump_value".to_owned(),
                    },
                    YirFunctionParameter {
                        name: "current".to_owned(),
                        ty: "i64".to_owned(),
                        ownership: YirValueOwnership::Value,
                        node: "bump_current".to_owned(),
                    },
                ],
                result: Some(YirFunctionResult {
                    ty: "Counter".to_owned(),
                    ownership: YirValueOwnership::Owned,
                    node: "bump_return".to_owned(),
                }),
                body_nodes: vec![
                    "bump_value".to_owned(),
                    "bump_current".to_owned(),
                    "bump_one".to_owned(),
                    "bump_next".to_owned(),
                    "bump_struct".to_owned(),
                    "bump_return".to_owned(),
                ],
            },
            YirFunction {
                name: "main".to_owned(),
                domain: "cpu".to_owned(),
                role: YirFunctionRole::Entry,
                parameters: Vec::new(),
                result: Some(YirFunctionResult {
                    ty: "i64".to_owned(),
                    ownership: YirValueOwnership::Value,
                    node: "main_return".to_owned(),
                }),
                body_nodes: vec![
                    "initial".to_owned(),
                    "limit".to_owned(),
                    "step".to_owned(),
                    "carry".to_owned(),
                    "loop".to_owned(),
                    "loop_result".to_owned(),
                    "main_return".to_owned(),
                ],
            },
        ]);

        let trace = execute_module(&module).expect("scoped aggregate loop should execute");
        assert_eq!(trace.values.get("loop"), Some(&Value::Int(3)));
        let Value::Struct(counter) = &trace.values["loop_result"] else {
            panic!("loop result should be a Counter struct");
        };
        assert_eq!(counter.fields, vec![("value".to_owned(), Value::Int(3))]);
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|event| event.contains("effect cpu.return_owned_struct"))
                .count(),
            3
        );
    }

    #[test]
    fn unselected_variant_field_error_still_fails_without_lazy_select() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("payload", "const_i64", &["41"]),
            cpu_node("err", "struct", &["Result.Err", "value=payload"]),
            cpu_node(
                "wrong_payload",
                "variant_field",
                &["err", "Result.Ok", "value"],
            ),
        ]);
        module
            .edges
            .extend([dep("payload", "err"), dep("err", "wrong_payload")]);

        let error = execute_module(&module).expect_err("standalone wrong variant must fail");
        assert!(error.contains("wrong_payload"));
        assert!(error.contains("expects variant `Result.Ok`"));
    }

    #[test]
    fn non_lazy_select_between_variants_preserves_union() {
        let mut module = YirModule::new("0.1");
        module.resources.push(cpu_resource());
        module.nodes.extend([
            cpu_node("cond", "const_bool", &["true"]),
            cpu_node("ok_payload", "const_i64", &["7"]),
            cpu_node("err_payload", "const_i64", &["99"]),
            cpu_node("ok", "struct", &["Result.Ok", "value=ok_payload"]),
            cpu_node("err", "struct", &["Result.Err", "value=err_payload"]),
            cpu_node("selected", "select", &["cond", "ok", "err"]),
            cpu_node("selected_is_ok", "variant_is", &["selected", "Result.Ok"]),
            cpu_node(
                "selected_ok_value",
                "variant_field",
                &["selected", "Result.Ok", "value"],
            ),
            cpu_node(
                "selected_err_value",
                "variant_field",
                &["selected", "Result.Err", "value"],
            ),
        ]);
        module.edges.extend([
            dep("ok_payload", "ok"),
            dep("err_payload", "err"),
            dep("cond", "selected"),
            dep("ok", "selected"),
            dep("err", "selected"),
            dep("selected", "selected_is_ok"),
            dep("selected", "selected_ok_value"),
            dep("selected", "selected_err_value"),
        ]);

        let trace = execute_module(&module).expect("variant select should execute");
        assert_eq!(trace.values.get("selected_is_ok"), Some(&Value::Bool(true)));
        assert_eq!(trace.values.get("selected_ok_value"), Some(&Value::Int(7)));
        assert_eq!(
            trace.values.get("selected_err_value"),
            Some(&Value::Int(99))
        );
    }
}
