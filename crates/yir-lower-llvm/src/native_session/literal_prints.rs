use std::collections::{BTreeMap, BTreeSet};

use super::function::FunctionAdmission;
use yir_core::{Node, YirFunction};

/// Static sites, not a per-node runtime fuel counter or a general effect capability.
pub const MAX_LITERAL_PRINT_SITES: usize = 64;

/// Explicit node grants for this emission only. Names confer no persistent trust:
/// the selected closure, literal operands, exact types and order are rechecked.
#[derive(Debug, Clone)]
pub struct LiteralPrintPolicy {
    sites: BTreeSet<String>,
}

impl LiteralPrintPolicy {
    pub(super) fn sites(&self) -> impl Iterator<Item = &str> {
        self.sites.iter().map(String::as_str)
    }

    pub fn new<I, S>(sites: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut selected = BTreeSet::new();
        for site in sites {
            let site = site.into();
            if site.is_empty() || !selected.insert(site) {
                return Err(
                    "native literal print policy has an empty or duplicate grant".to_owned(),
                );
            }
            if selected.len() > MAX_LITERAL_PRINT_SITES {
                return Err("native literal print policy exceeds its static site bound".to_owned());
            }
        }
        Ok(Self { sites: selected })
    }

    pub(super) fn permit(
        &self,
        node: &Node,
        nodes: &BTreeMap<&str, &Node>,
    ) -> Result<bool, String> {
        if !self.sites.contains(&node.name) {
            return Ok(false);
        }
        let operand = match (node.op.instruction.as_str(), node.op.args.as_slice()) {
            ("print", [value]) | ("guard_print", [_, value]) => value,
            _ => return Err("native literal print grant targets an unsupported effect".to_owned()),
        };
        if !nodes.get(operand.as_str()).is_some_and(|value| {
            value.op.module == "cpu"
                && matches!(value.op.instruction.as_str(), "const" | "const_i64")
                && value.op.args.len() == 1
                && value.op.args[0].parse::<i64>().is_ok()
        }) {
            return Err(format!(
                "native literal print `{}` requires a direct i64 constant",
                node.name
            ));
        }
        Ok(true)
    }

    pub(super) fn selected_sites(
        &self,
        selected: &BTreeSet<String>,
    ) -> Result<Vec<String>, String> {
        if !self.sites.is_subset(selected) {
            return Err("native literal print policy contains an unselected grant".to_owned());
        }
        Ok(self.sites.iter().cloned().collect())
    }

    pub(super) fn validate_closure(
        &self,
        functions: &BTreeMap<&str, YirFunction>,
        graph: &BTreeMap<String, BTreeSet<String>>,
        admission: &FunctionAdmission<'_>,
    ) -> Result<(), String> {
        let mut effectful = functions
            .values()
            .filter(|function| {
                function
                    .body_nodes
                    .iter()
                    .any(|name| self.sites.contains(name))
            })
            .map(|function| function.name.clone())
            .collect::<BTreeSet<_>>();
        loop {
            let before = effectful.len();
            for (caller, callees) in graph {
                if callees.iter().any(|callee| effectful.contains(callee)) {
                    effectful.insert(caller.clone());
                }
            }
            if effectful.len() == before {
                break;
            }
        }
        let functions = functions
            .iter()
            .map(|(name, function)| (*name, function))
            .collect::<BTreeMap<_, _>>();
        for function in functions.values() {
            self.validate_order(function, admission, &functions, &effectful)?;
        }
        Ok(())
    }

    fn validate_order(
        &self,
        function: &YirFunction,
        admission: &FunctionAdmission<'_>,
        functions: &BTreeMap<&str, &YirFunction>,
        effectful: &BTreeSet<String>,
    ) -> Result<(), String> {
        let sites = function
            .body_nodes
            .iter()
            .map(|name| {
                let callee = super::calls::target(
                    admission.nodes[name.as_str()],
                    functions,
                    &admission.nodes,
                )?;
                Ok((self.sites.contains(name)
                    || callee.is_some_and(|f| effectful.contains(&f.name)))
                .then_some(name))
            })
            .collect::<Result<Vec<_>, String>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        if sites.is_empty() {
            return Ok(());
        }
        let mut dependencies = BTreeMap::<&str, BTreeSet<&str>>::new();
        let mut sensitive = Vec::new();
        for name in &function.body_nodes {
            let node = admission.nodes[name.as_str()];
            let semantics = admission
                .registry
                .lookup("cpu")
                .unwrap()
                .describe(node, admission.resources[node.resource.as_str()])?;
            let deps = dependencies.entry(name).or_default();
            // Use stored node arguments, not temporary semantic dependency strings.
            for dependency in &semantics.dependencies {
                let stored = admission
                    .nodes
                    .get_key_value(dependency.as_str())
                    .ok_or("native literal print order has an unknown dependency")?
                    .0;
                deps.insert(*stored);
            }
            deps.extend(
                admission
                    .incoming
                    .get(name.as_str())
                    .into_iter()
                    .flatten()
                    .copied(),
            );
            let op = node.op.instruction.as_str();
            if semantics.has_effect
                || op.starts_with("call_")
                || op.starts_with("loop_")
                || op.starts_with("return_")
                || matches!(op, "guard_return" | "div" | "rem")
            {
                sensitive.push(name.as_str());
            }
        }
        let mut dependents = BTreeMap::<&str, BTreeSet<&str>>::new();
        for (name, inputs) in &dependencies {
            for input in inputs {
                dependents.entry(input).or_default().insert(name);
            }
        }
        for site in sites {
            // Two bounded traversals per granted site, never an all-pairs matrix.
            let before = reachable(site, &dependencies);
            let after = reachable(site, &dependents);
            for other in &sensitive {
                if site != other && !before.contains(other) && !after.contains(other) {
                    return Err(format!(
                        "native literal print `{site}` lacks dependency order with `{other}`"
                    ));
                }
            }
        }
        Ok(())
    }
}

fn reachable<'a>(name: &'a str, graph: &BTreeMap<&'a str, BTreeSet<&'a str>>) -> BTreeSet<&'a str> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![name];
    while let Some(current) = pending.pop() {
        for next in graph.get(current).into_iter().flatten() {
            if seen.insert(*next) {
                pending.push(next);
            }
        }
    }
    seen
}
