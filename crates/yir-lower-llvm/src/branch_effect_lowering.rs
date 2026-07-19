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
}

pub(crate) fn lower_cpu_branch_effect_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
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
