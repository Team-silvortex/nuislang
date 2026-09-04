use yir_core::{
    BranchEffectAccess, BranchEffectAction, BranchEffectActionCapability, BranchEffectResult,
    ExecutionState, Node, Resource, Value,
};

use crate::runtime_helpers::unwrap_present_frame_payload;

const POINTER_READ: &[BranchEffectAccess] = &[BranchEffectAccess::ResourceRead];
const POINTER_OWN: &[BranchEffectAccess] = &[BranchEffectAccess::ResourceOwn];
const VALUE_READ: &[BranchEffectAccess] = &[BranchEffectAccess::ValueRead];
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
        instruction: "present_frame",
        result: BranchEffectResult::Unit,
        operand_accesses: VALUE_READ,
    },
    BranchEffectActionCapability {
        module: "cpu",
        instruction: "take_ptr_drop_other",
        result: BranchEffectResult::OwnedPointer,
        operand_accesses: POINTER_SELECT_OWN,
    },
    BranchEffectActionCapability {
        module: "cpu",
        instruction: yir_core::ffi::OWNED_BUFFER_BRANCH_TRANSFER_ACTION,
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
    match action.instruction {
        "load_value" => {
            let pointer = state.expect_pointer(action.operands[0].value)?;
            let value = state.read_heap_node(pointer)?.value;
            state.push_resource_event(
                resource,
                format!("effect {} load_value {pointer:?}", parent.op.full_name()),
            );
            Ok(Value::Int(value))
        }
        "free" => {
            let pointer = state.expect_pointer(action.operands[0].value)?;
            state.free_heap_node(pointer)?;
            state.push_resource_event(
                resource,
                format!("effect {} free {pointer:?}", parent.op.full_name()),
            );
            Ok(Value::Unit)
        }
        "present_frame" => {
            let frame =
                unwrap_present_frame_payload(state.expect_value(action.operands[0].value)?.clone());
            if let Value::Frame(surface) = &frame {
                state.record_presented_frame(surface.clone());
            }
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.present_frame @{} [{}]: {}",
                    parent.resource, resource.kind.raw, frame
                ),
            );
            Ok(Value::Unit)
        }
        "take_ptr_drop_other" => {
            let pointer = state.expect_pointer(action.operands[0].value)?;
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
        instruction if instruction == yir_core::ffi::OWNED_BUFFER_BRANCH_TRANSFER_ACTION => {
            let pointer = state.expect_pointer(action.operands[0].value)?;
            let discarded = state.expect_pointer(action.operands[1].value)?;
            if pointer == discarded {
                return Err(format!(
                    "{} cannot transfer and discard the same owned buffer",
                    parent.op.full_name()
                ));
            }
            state.free_heap_node(discarded)?;
            state.push_resource_event(
                resource,
                format!(
                    "effect {} {} selected={pointer:?} discarded={discarded:?}",
                    parent.op.full_name(),
                    yir_core::ffi::OWNED_BUFFER_BRANCH_TRANSFER_ACTION
                ),
            );
            Ok(Value::Pointer(pointer))
        }
        instruction => Err(format!(
            "CpuMod does not implement registered branch action `{instruction}`"
        )),
    }
}
