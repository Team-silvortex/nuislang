use super::*;

pub(super) struct EntryExits<'a> {
    owners: BTreeMap<&'a str, &'a YirFunction>,
    closed: BTreeSet<&'a str>,
}

impl<'a> EntryExits<'a> {
    pub(super) fn new(module: &'a YirModule) -> Self {
        Self {
            owners: module
                .functions
                .iter()
                .filter(|function| function.role == YirFunctionRole::Entry)
                .flat_map(|function| {
                    function
                        .body_nodes
                        .iter()
                        .map(move |name| (name.as_str(), function))
                })
                .collect(),
            closed: BTreeSet::new(),
        }
    }

    pub(super) fn admits(&self, node: &str) -> bool {
        self.owners
            .get(node)
            .is_some_and(|owner| !self.closed.contains(owner.name.as_str()))
    }

    pub(super) fn observe(
        &mut self,
        name: &str,
        engine: &mut ExecutionEngine<'_>,
        delayed: &mut BTreeMap<String, String>,
    ) -> Result<(), String> {
        let Some(owner) = self.owners.get(name) else {
            return Ok(());
        };
        let node = engine.nodes_by_name[name];
        let domain = engine.registry.lookup(&node.op.module).ok_or_else(|| {
            format!(
                "node `{name}` references unregistered mod `{}`",
                node.op.module
            )
        })?;
        if let Some(value) =
            domain.function_exit(node, engine.resources[&node.resource], &engine.state)?
        {
            // Keep the graph's published result identity, but do not execute
            // this entry's tail or suppress another entry/global node.
            if let Some(result) = &owner.result {
                engine.state.values.insert(result.node.clone(), value);
            }
            self.closed.insert(owner.name.as_str());
            for node in &owner.body_nodes {
                delayed.remove(node);
            }
        }
        Ok(())
    }
}
