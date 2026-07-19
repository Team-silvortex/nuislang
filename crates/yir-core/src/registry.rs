use std::collections::BTreeMap;

use crate::{
    BranchEffectAction, BranchEffectActionCapability, ExecutionState, Node,
    PlannedBranchEffectAction, PlannedBranchEffectOperand, Resource, Value,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionSemantics {
    pub dependencies: Vec<String>,
    pub has_effect: bool,
}

impl InstructionSemantics {
    pub fn pure(dependencies: Vec<String>) -> Self {
        Self {
            dependencies,
            has_effect: false,
        }
    }

    pub fn effect(dependencies: Vec<String>) -> Self {
        Self {
            dependencies,
            has_effect: true,
        }
    }
}

pub trait RegisteredMod: Send + Sync {
    fn module_name(&self) -> &'static str;

    fn branch_effect_action_capabilities(&self) -> &'static [BranchEffectActionCapability] {
        &[]
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String>;

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String>;

    fn execute_branch_effect_action(
        &self,
        action: &BranchEffectAction<'_>,
        _parent: &Node,
        _resource: &Resource,
        _state: &mut ExecutionState,
    ) -> Result<Value, String> {
        Err(format!(
            "registered mod `{}` does not implement branch action `{}`",
            self.module_name(),
            action.instruction
        ))
    }
}

#[derive(Default)]
pub struct ModRegistry {
    mods: BTreeMap<String, Box<dyn RegisteredMod>>,
}

impl ModRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<M>(&mut self, module: M)
    where
        M: RegisteredMod + 'static,
    {
        self.mods
            .insert(module.module_name().to_owned(), Box::new(module));
    }

    pub fn lookup(&self, name: &str) -> Option<&dyn RegisteredMod> {
        self.mods.get(name).map(|module| module.as_ref())
    }

    pub fn branch_effect_action_capability(
        &self,
        module: &str,
        instruction: &str,
    ) -> Option<&'static BranchEffectActionCapability> {
        self.lookup(module)?
            .branch_effect_action_capabilities()
            .iter()
            .find(|capability| capability.instruction == instruction)
    }

    pub fn plan_branch_effect_action(
        &self,
        module: &str,
        instruction: &str,
        operand_values: Vec<String>,
    ) -> Result<PlannedBranchEffectAction, String> {
        let capability = self
            .branch_effect_action_capability(module, instruction)
            .ok_or_else(|| format!("unregistered branch action `{module}.{instruction}`"))?;
        if operand_values.len() != capability.operand_accesses.len() {
            return Err(format!(
                "branch action `{module}.{instruction}` expects {} operands, got {}",
                capability.operand_accesses.len(),
                operand_values.len()
            ));
        }
        let operands = capability
            .operand_accesses
            .iter()
            .copied()
            .zip(operand_values)
            .map(|(access, value)| PlannedBranchEffectOperand { access, value })
            .collect();
        Ok(PlannedBranchEffectAction {
            module: capability.module.to_owned(),
            instruction: capability.instruction.to_owned(),
            result: capability.result,
            operands,
        })
    }
}
