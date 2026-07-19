use super::support::*;
use crate::{
    default_branch_effect_llvm_emitters, emit_module_with_registries, BranchEffectLlvmEmitContext,
    BranchEffectLlvmEmitterRegistry, BranchEffectLlvmValue,
};
use yir_core::{
    BranchEffectAction, BranchEffectActionCapability, BranchEffectResult, ExecutionState,
    InstructionSemantics, ModRegistry, RegisteredMod, Value,
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
}

fn emit_probe_value(
    action: &BranchEffectAction<'_>,
    _node: &Node,
    context: &mut BranchEffectLlvmEmitContext<'_>,
) -> Result<BranchEffectLlvmValue, String> {
    let value = match action.instruction {
        "left" => 41,
        "right" => 73,
        other => return Err(format!("unknown probe emitter `{other}`")),
    };
    let result = context.fresh_register();
    context.push(format!("  {result} = add i64 0, {value}"));
    Ok(BranchEffectLlvmValue::I64(result))
}

fn probe_emitter_registry() -> BranchEffectLlvmEmitterRegistry {
    let mut registry = default_branch_effect_llvm_emitters();
    registry.register("probe", "left", emit_probe_value);
    registry.register("probe", "right", emit_probe_value);
    registry
}

#[test]
fn registered_non_cpu_action_emitters_join_branch_composition() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "choose", "cpu.const_bool", vec!["false"]);
    push_cpu_node(
        &mut module,
        "selected",
        "cpu.branch_effect",
        vec![
            "choose", "i64", "1", "probe", "left", "i64", "0", "1", "probe", "right", "i64", "0",
        ],
    );
    push_dep(&mut module, "choose", "selected");

    let mut yir_registry: ModRegistry = yir_verify::default_registry();
    yir_registry.register(ProbeMod);
    let llvm_emitters = probe_emitter_registry();
    let llvm = emit_module_with_registries(&module, &yir_registry, &llvm_emitters)
        .expect("registered probe emitters should lower");

    assert!(llvm.contains("add i64 0, 41"));
    assert!(llvm.contains("add i64 0, 73"));
    assert!(llvm.contains("phi i64"));

    let error = emit_module_with_registries(
        &module,
        &yir_registry,
        &default_branch_effect_llvm_emitters(),
    )
    .unwrap_err();
    assert!(error.contains("no LLVM emitter for `probe.left`"));
}

#[test]
fn owned_pointer_actions_drop_the_unselected_owner_and_merge_with_phi() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "choose", "cpu.const_bool", vec!["false"]);
    push_cpu_const_i64(&mut module, "left_value", "41");
    push_cpu_const_i64(&mut module, "right_value", "73");
    push_cpu_node(&mut module, "nil", "cpu.null", vec![]);
    push_cpu_node(
        &mut module,
        "left",
        "cpu.alloc_node",
        vec!["left_value", "nil"],
    );
    push_cpu_node(
        &mut module,
        "right",
        "cpu.alloc_node",
        vec!["right_value", "nil"],
    );
    push_cpu_node(
        &mut module,
        "selected",
        "cpu.branch_effect",
        vec![
            "choose",
            "owned_ptr",
            "1",
            "cpu",
            "take_ptr_drop_other",
            "owned_ptr",
            "2",
            "resource_own",
            "left",
            "resource_own",
            "right",
            "1",
            "cpu",
            "take_ptr_drop_other",
            "owned_ptr",
            "2",
            "resource_own",
            "right",
            "resource_own",
            "left",
        ],
    );
    push_cpu_node(&mut module, "drop_selected", "cpu.free", vec!["selected"]);
    push_deps(
        &mut module,
        &[
            ("left_value", "left"),
            ("nil", "left"),
            ("right_value", "right"),
            ("nil", "right"),
            ("choose", "selected"),
            ("left", "selected"),
            ("right", "selected"),
            ("selected", "drop_selected"),
        ],
    );
    for (from, to) in [
        ("left", "selected"),
        ("right", "selected"),
        ("selected", "drop_selected"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Lifetime,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm = emit_module(&module).expect("owned pointer branch should lower");
    assert!(llvm.contains("phi ptr"));
    assert_eq!(llvm.matches("call void @free(ptr").count(), 3);
}
