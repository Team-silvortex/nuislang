use std::collections::BTreeMap;

use yir_core::{
    branch_effect_merge_is_valid, parse_branch_effect_args, BranchEffectAccess, BranchEffectAction,
    BranchEffectResult, Node,
};

use super::{
    fresh_block, fresh_reg,
    value_ref::{coerce_to_i64, get_ptr},
    LlvmValueRef,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchEffectLlvmValue {
    Unit,
    I64(String),
    OwnedPointer(String),
    OwnedExternalBuffer {
        ptr: String,
        len: String,
        abi: String,
        destructor: String,
        destructor_signature_hash: String,
    },
}

pub struct BranchEffectLlvmEmitContext<'a> {
    body: &'a mut Vec<String>,
    registers: &'a BTreeMap<String, LlvmValueRef>,
    next_reg: &'a mut usize,
}

impl BranchEffectLlvmEmitContext<'_> {
    pub fn pointer_operand(
        &self,
        action: &BranchEffectAction<'_>,
        index: usize,
    ) -> Result<String, String> {
        action
            .operands
            .get(index)
            .and_then(|operand| get_ptr(self.registers, operand.value))
            .map(str::to_owned)
            .ok_or_else(|| {
                format!(
                    "cannot resolve pointer operand {index} for `{}.{}`",
                    action.module, action.instruction
                )
            })
    }

    pub fn owned_external_buffer_operand(
        &self,
        action: &BranchEffectAction<'_>,
        index: usize,
    ) -> Result<(String, String, String, String, String), String> {
        let value = action
            .operands
            .get(index)
            .and_then(|operand| self.registers.get(operand.value));
        match value {
            Some(LlvmValueRef::OwnedExternalBuffer {
                ptr,
                len,
                abi,
                destructor,
                destructor_signature_hash,
            }) => Ok((
                ptr.clone(),
                len.clone(),
                abi.clone(),
                destructor.clone(),
                destructor_signature_hash.clone(),
            )),
            _ => Err(format!(
                "cannot resolve registered owned-buffer operand {index} for `{}.{}`",
                action.module, action.instruction
            )),
        }
    }

    pub fn fresh_register(&mut self) -> String {
        fresh_reg(self.next_reg)
    }

    pub fn push(&mut self, instruction: impl Into<String>) {
        self.body.push(instruction.into());
    }
}

pub type BranchEffectLlvmEmitter =
    for<'action, 'context> fn(
        &BranchEffectAction<'action>,
        &Node,
        &mut BranchEffectLlvmEmitContext<'context>,
    ) -> Result<BranchEffectLlvmValue, String>;

#[derive(Default)]
pub struct BranchEffectLlvmEmitterRegistry {
    emitters: BTreeMap<(String, String), BranchEffectLlvmEmitter>,
}

impl BranchEffectLlvmEmitterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        module: impl Into<String>,
        instruction: impl Into<String>,
        emitter: BranchEffectLlvmEmitter,
    ) {
        self.emitters
            .insert((module.into(), instruction.into()), emitter);
    }

    pub fn contains(&self, module: &str, instruction: &str) -> bool {
        self.emitters
            .contains_key(&(module.to_owned(), instruction.to_owned()))
    }

    fn emitter(&self, action: &BranchEffectAction<'_>) -> Option<BranchEffectLlvmEmitter> {
        self.emitters
            .get(&(action.module.to_owned(), action.instruction.to_owned()))
            .copied()
    }
}

pub fn default_branch_effect_llvm_emitters() -> BranchEffectLlvmEmitterRegistry {
    let mut registry = BranchEffectLlvmEmitterRegistry::new();
    register_cpu_branch_effect_llvm_emitters(&mut registry);
    registry
}

pub fn register_cpu_branch_effect_llvm_emitters(registry: &mut BranchEffectLlvmEmitterRegistry) {
    registry.register("cpu", "load_value", emit_cpu_load_value);
    registry.register("cpu", "free", emit_cpu_free);
    registry.register("cpu", "take_ptr_drop_other", emit_cpu_take_ptr_drop_other);
    registry.register(
        "cpu",
        yir_core::ffi::OWNED_BUFFER_BRANCH_TRANSFER_ACTION,
        emit_cpu_take_owned_buffer_drop_other,
    );
}

pub(crate) fn lower_cpu_branch_effect_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    buffer_lengths: &mut BTreeMap<String, String>,
    next_reg: &mut usize,
    next_block: &mut usize,
    emitter_registry: &BranchEffectLlvmEmitterRegistry,
) -> Result<bool, String> {
    if node.op.module != "cpu" || node.op.instruction != "branch_effect" {
        return Ok(false);
    }
    let args = parse_branch_effect_args(&node.op.args)
        .ok_or_else(|| format!("cpu.branch_effect `{}` has invalid arguments", node.name))?;
    if !branch_effect_merge_is_valid(&args) {
        return Err(format!(
            "cpu.branch_effect `{}` actions do not produce the declared {:?} merge result",
            node.name, args.merge_result
        ));
    }
    let condition = registers.get(args.condition).cloned().ok_or_else(|| {
        format!(
            "cpu.branch_effect `{}` cannot resolve condition `{}`",
            node.name, args.condition
        )
    })?;
    let condition = coerce_to_i64(&condition, body, next_reg)
        .ok_or_else(|| format!("cpu.branch_effect `{}` condition is not scalar", node.name))?;
    let condition_i1 = fresh_reg(next_reg);
    body.push(format!("  {condition_i1} = icmp ne i64 {condition}, 0"));
    let then_label = fresh_block(next_block, "branch_effect_then");
    let else_label = fresh_block(next_block, "branch_effect_else");
    let merge_label = fresh_block(next_block, "branch_effect_merge");
    body.push(format!(
        "  br i1 {condition_i1}, label %{then_label}, label %{else_label}"
    ));
    body.push(format!("{then_label}:"));
    let then_result = emit_actions(
        &args.then_actions,
        node,
        body,
        registers,
        next_reg,
        emitter_registry,
    )?;
    body.push(format!("  br label %{merge_label}"));
    body.push(format!("{else_label}:"));
    let else_result = emit_actions(
        &args.else_actions,
        node,
        body,
        registers,
        next_reg,
        emitter_registry,
    )?;
    body.push(format!("  br label %{merge_label}"));
    body.push(format!("{merge_label}:"));
    let merged = match (args.merge_result, then_result, else_result) {
        (BranchEffectResult::Unit, _, _) => LlvmValueRef::Void,
        (
            BranchEffectResult::I64,
            BranchEffectLlvmValue::I64(then_value),
            BranchEffectLlvmValue::I64(else_value),
        ) => {
            let merged = fresh_reg(next_reg);
            body.push(format!(
                "  {merged} = phi i64 [{then_value}, %{then_label}], [{else_value}, %{else_label}]"
            ));
            LlvmValueRef::I64(merged)
        }
        (
            BranchEffectResult::OwnedPointer,
            BranchEffectLlvmValue::OwnedPointer(then_value),
            BranchEffectLlvmValue::OwnedPointer(else_value),
        ) => {
            let merged = fresh_reg(next_reg);
            body.push(format!(
                "  {merged} = phi ptr [{then_value}, %{then_label}], [{else_value}, %{else_label}]"
            ));
            LlvmValueRef::Ptr(merged)
        }
        (
            BranchEffectResult::OwnedPointer,
            BranchEffectLlvmValue::OwnedExternalBuffer {
                ptr: then_ptr,
                len: then_len,
                abi: then_abi,
                destructor: then_destructor,
                destructor_signature_hash: then_destructor_signature_hash,
            },
            BranchEffectLlvmValue::OwnedExternalBuffer {
                ptr: else_ptr,
                len: else_len,
                abi: else_abi,
                destructor: else_destructor,
                destructor_signature_hash: else_destructor_signature_hash,
            },
        ) if then_abi == else_abi
            && then_destructor == else_destructor
            && then_destructor_signature_hash == else_destructor_signature_hash =>
        {
            let merged_ptr = fresh_reg(next_reg);
            body.push(format!(
                "  {merged_ptr} = phi ptr [{then_ptr}, %{then_label}], [{else_ptr}, %{else_label}]"
            ));
            let merged_len = fresh_reg(next_reg);
            body.push(format!(
                "  {merged_len} = phi i64 [{then_len}, %{then_label}], [{else_len}, %{else_label}]"
            ));
            buffer_lengths.insert(node.name.clone(), merged_len.clone());
            LlvmValueRef::OwnedExternalBuffer {
                ptr: merged_ptr,
                len: merged_len,
                abi: then_abi,
                destructor: then_destructor,
                destructor_signature_hash: then_destructor_signature_hash,
            }
        }
        (result, _, _) => {
            return Err(format!(
                "cpu.branch_effect `{}` emitter results do not satisfy {result:?} merge",
                node.name
            ));
        }
    };
    registers.insert(node.name.clone(), merged);
    Ok(true)
}

fn emit_actions(
    actions: &[BranchEffectAction<'_>],
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    emitter_registry: &BranchEffectLlvmEmitterRegistry,
) -> Result<BranchEffectLlvmValue, String> {
    let mut result = BranchEffectLlvmValue::Unit;
    for action in actions {
        let emitter = emitter_registry.emitter(action).ok_or_else(|| {
            format!(
                "cpu.branch_effect `{}` has no LLVM emitter for `{}.{}`",
                node.name, action.module, action.instruction
            )
        })?;
        let mut context = BranchEffectLlvmEmitContext {
            body,
            registers,
            next_reg,
        };
        result = emitter(action, node, &mut context)?;
    }
    Ok(result)
}

fn emit_cpu_load_value(
    action: &BranchEffectAction<'_>,
    _node: &Node,
    context: &mut BranchEffectLlvmEmitContext<'_>,
) -> Result<BranchEffectLlvmValue, String> {
    if action.result != BranchEffectResult::I64
        || !matches!(action.operands.as_slice(), [operand] if operand.access == BranchEffectAccess::ResourceRead)
    {
        return Err("cpu.load_value branch action has an incompatible contract".to_owned());
    }
    let pointer = context.pointer_operand(action, 0)?;
    let slot = context.fresh_register();
    context.push(format!(
        "  {slot} = getelementptr inbounds %cpu.node, ptr {pointer}, i32 0, i32 0"
    ));
    let loaded = context.fresh_register();
    context.push(format!("  {loaded} = load i64, ptr {slot}"));
    Ok(BranchEffectLlvmValue::I64(loaded))
}

fn emit_cpu_free(
    action: &BranchEffectAction<'_>,
    _node: &Node,
    context: &mut BranchEffectLlvmEmitContext<'_>,
) -> Result<BranchEffectLlvmValue, String> {
    if action.result != BranchEffectResult::Unit
        || !matches!(action.operands.as_slice(), [operand] if operand.access == BranchEffectAccess::ResourceOwn)
    {
        return Err("cpu.free branch action has an incompatible contract".to_owned());
    }
    let pointer = context.pointer_operand(action, 0)?;
    context.push(format!("  call void @free(ptr {pointer})"));
    Ok(BranchEffectLlvmValue::Unit)
}

fn emit_cpu_take_ptr_drop_other(
    action: &BranchEffectAction<'_>,
    _node: &Node,
    context: &mut BranchEffectLlvmEmitContext<'_>,
) -> Result<BranchEffectLlvmValue, String> {
    if action.result != BranchEffectResult::OwnedPointer
        || !matches!(
            action.operands.as_slice(),
            [selected, discarded]
                if selected.access == BranchEffectAccess::ResourceOwn
                    && discarded.access == BranchEffectAccess::ResourceOwn
        )
    {
        return Err(
            "cpu.take_ptr_drop_other branch action has an incompatible contract".to_owned(),
        );
    }
    let selected = context.pointer_operand(action, 0)?;
    let discarded = context.pointer_operand(action, 1)?;
    context.push(format!("  call void @free(ptr {discarded})"));
    Ok(BranchEffectLlvmValue::OwnedPointer(selected))
}

fn emit_cpu_take_owned_buffer_drop_other(
    action: &BranchEffectAction<'_>,
    _node: &Node,
    context: &mut BranchEffectLlvmEmitContext<'_>,
) -> Result<BranchEffectLlvmValue, String> {
    if action.result != BranchEffectResult::OwnedPointer
        || !matches!(
            action.operands.as_slice(),
            [selected, discarded]
                if selected.access == BranchEffectAccess::ResourceOwn
                    && discarded.access == BranchEffectAccess::ResourceOwn
        )
    {
        return Err(format!(
            "cpu.{} branch action has an incompatible contract",
            yir_core::ffi::OWNED_BUFFER_BRANCH_TRANSFER_ACTION
        ));
    }
    let (
        selected_ptr,
        selected_len,
        selected_abi,
        selected_destructor,
        selected_destructor_signature_hash,
    ) = context.owned_external_buffer_operand(action, 0)?;
    let (
        discarded_ptr,
        _,
        discarded_abi,
        discarded_destructor,
        discarded_destructor_signature_hash,
    ) = context.owned_external_buffer_operand(action, 1)?;
    if selected_abi != discarded_abi
        || selected_destructor != discarded_destructor
        || selected_destructor_signature_hash != discarded_destructor_signature_hash
    {
        return Err(format!(
            "cpu.{} requires one exact ABI/destructor/hash identity",
            yir_core::ffi::OWNED_BUFFER_BRANCH_TRANSFER_ACTION
        ));
    }
    let status = context.fresh_register();
    context.push(format!(
        "  {status} = call i64 @{discarded_destructor}(ptr {discarded_ptr})"
    ));
    Ok(BranchEffectLlvmValue::OwnedExternalBuffer {
        ptr: selected_ptr,
        len: selected_len,
        abi: selected_abi,
        destructor: selected_destructor,
        destructor_signature_hash: selected_destructor_signature_hash,
    })
}
