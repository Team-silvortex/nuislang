use std::{collections::BTreeSet, sync::Arc};

use yir_core::{
    BranchEffectAction, BranchEffectActionCapability, ExecutionState, InstructionSemantics,
    ModRegistry, Node, ProviderCompletionRegistration, RegisteredMod, Resource, Value, YirModule,
};

/// The packaged host explicitly admits CPU orchestration, not an implicit
/// reference GPU/network fallback. Other registered semantics remain available
/// for verification of the embedded child, but cannot execute in this context.
pub(super) fn cpu_parent_registry(module: &YirModule) -> Result<ModRegistry, String> {
    let base = Arc::new(yir_verify::default_registry());
    let mut registry = ModRegistry::new();
    for name in module
        .nodes
        .iter()
        .map(|node| &node.op.module)
        .collect::<BTreeSet<_>>()
    {
        let registered = base
            .lookup(name)
            .ok_or_else(|| format!("unregistered module `{name}`"))?;
        registry.register(HostParentMod {
            name: registered.module_name(),
            base: base.clone(),
        });
    }
    Ok(registry)
}

struct HostParentMod {
    name: &'static str,
    base: Arc<ModRegistry>,
}

impl HostParentMod {
    fn admitted(&self) -> Result<&dyn RegisteredMod, String> {
        if self.name != "cpu" {
            return Err(format!(
                "packaged parent CPU profile cannot execute `{}`",
                self.name
            ));
        }
        Ok(self.base.lookup(self.name).unwrap())
    }
}

impl RegisteredMod for HostParentMod {
    fn module_name(&self) -> &'static str {
        self.name
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        self.base
            .lookup(self.name)
            .unwrap()
            .describe(node, resource)
    }

    fn provider_completion_registration(
        &self,
        node: &Node,
    ) -> Option<ProviderCompletionRegistration> {
        self.base
            .lookup(self.name)
            .unwrap()
            .provider_completion_registration(node)
    }

    fn branch_effect_action_capabilities(&self) -> &'static [BranchEffectActionCapability] {
        self.base
            .lookup(self.name)
            .unwrap()
            .branch_effect_action_capabilities()
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        self.admitted()?.execute(node, resource, state)
    }

    fn execute_branch_effect_action(
        &self,
        action: &BranchEffectAction<'_>,
        parent: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        self.admitted()?
            .execute_branch_effect_action(action, parent, resource, state)
    }
}
