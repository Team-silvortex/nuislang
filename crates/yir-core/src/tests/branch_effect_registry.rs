use crate::*;

const NO_OPERANDS: &[BranchEffectAccess] = &[];
const PROBE_ACTIONS: &[BranchEffectActionCapability] = &[
    BranchEffectActionCapability {
        module: "probe",
        instruction: "left",
        result: BranchEffectResult::I64,
        operand_accesses: NO_OPERANDS,
    },
    BranchEffectActionCapability {
        module: "probe",
        instruction: "right",
        result: BranchEffectResult::I64,
        operand_accesses: NO_OPERANDS,
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

    fn describe(&self, node: &Node, _resource: &Resource) -> Result<InstructionSemantics, String> {
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
        parent: &Node,
        _resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        state.events.push(format!(
            "{} dispatched probe.{}",
            parent.op.full_name(),
            action.instruction
        ));
        match action.instruction {
            "left" => Ok(Value::Int(41)),
            "right" => Ok(Value::Int(73)),
            other => Err(format!("unknown probe action `{other}`")),
        }
    }
}

#[test]
fn registry_dispatches_selected_action_to_its_owning_mod() {
    let mut registry = ModRegistry::new();
    registry.register(ProbeMod);
    let node = Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.branch_effect",
            [
                "choose", "i64", "1", "probe", "left", "i64", "0", "1", "probe", "right", "i64",
                "0",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let resource = Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    };
    let mut state = ExecutionState::default();
    state.values.insert("choose".to_owned(), Value::Bool(false));

    assert!(registry
        .describe_branch_effect_node(&node)
        .unwrap()
        .is_some());
    assert_eq!(
        registry
            .execute_branch_effect_node(&node, &resource, &mut state)
            .unwrap(),
        Some(Value::Int(73))
    );
    assert_eq!(state.events, ["cpu.branch_effect dispatched probe.right"]);
}
