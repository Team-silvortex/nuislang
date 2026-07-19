use yir_core::{
    BranchEffectAccess, BranchEffectAction, BranchEffectActionCapability, BranchEffectResult,
    ExecutionState, Node, Resource, Value,
};

const POINTER_READ: &[BranchEffectAccess] = &[BranchEffectAccess::ResourceRead];
const POINTER_OWN: &[BranchEffectAccess] = &[BranchEffectAccess::ResourceOwn];
const POINTER_SELECT_OWN: &[BranchEffectAccess] = &[
    BranchEffectAccess::ResourceOwn,
    BranchEffectAccess::ResourceOwn,
];

pub(super) const CPU_BRANCH_EFFECT_ACTIONS: &[BranchEffectActionCapability] = &[
    BranchEffectActionCapability {
        module: "cpu",
        instruction: "load_value",
        result: BranchEffectResult::I64,
        operand_accesses: POINTER_READ,
    },
    BranchEffectActionCapability {
        module: "cpu",
        instruction: "free",
        result: BranchEffectResult::Unit,
        operand_accesses: POINTER_OWN,
    },
    BranchEffectActionCapability {
        module: "cpu",
        instruction: "take_ptr_drop_other",
        result: BranchEffectResult::OwnedPointer,
        operand_accesses: POINTER_SELECT_OWN,
    },
];

pub(super) fn execute_cpu_branch_effect_action(
    action: &BranchEffectAction<'_>,
    parent: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    let pointer = state.expect_pointer(action.operands[0].value)?;
    match action.instruction {
        "load_value" => {
            let value = state.read_heap_node(pointer)?.value;
            state.push_resource_event(
                resource,
                format!("effect {} load_value {pointer:?}", parent.op.full_name()),
            );
            Ok(Value::Int(value))
        }
        "free" => {
            state.free_heap_node(pointer)?;
            state.push_resource_event(
                resource,
                format!("effect {} free {pointer:?}", parent.op.full_name()),
            );
            Ok(Value::Unit)
        }
        "take_ptr_drop_other" => {
            let discarded = state.expect_pointer(action.operands[1].value)?;
            if pointer == discarded {
                return Err(format!(
                    "{} cannot select and discard the same pointer",
                    parent.op.full_name()
                ));
            }
            state.free_heap_node(discarded)?;
            state.push_resource_event(
                resource,
                format!(
                    "effect {} take_ptr_drop_other selected={pointer:?} discarded={discarded:?}",
                    parent.op.full_name()
                ),
            );
            Ok(Value::Pointer(pointer))
        }
        instruction => Err(format!(
            "CpuMod does not implement registered branch action `{instruction}`"
        )),
    }
}
