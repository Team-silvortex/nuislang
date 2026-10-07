use std::collections::{BTreeMap, BTreeSet};

use super::{admission::admitted, LiteralPrintPolicy, ScalarKind, MAX_SCALAR_SLOTS};
use yir_core::{ModRegistry, Node, Resource, YirFunction, YirModule, YirValueOwnership};

pub(super) struct FunctionAdmission<'a> {
    pub module: &'a YirModule,
    pub nodes: BTreeMap<&'a str, &'a Node>,
    pub resources: BTreeMap<&'a str, &'a Resource>,
    pub incoming: BTreeMap<&'a str, Vec<&'a str>>,
    pub registry: &'a ModRegistry,
    pub literal_prints: Option<&'a LiteralPrintPolicy>,
}

impl<'a> FunctionAdmission<'a> {
    pub fn validate(&self, function: &YirFunction, callback: bool) -> Result<(), String> {
        if function.domain != "cpu" || !symbol_name(&function.name) {
            return Err(format!(
                "native scalar bridge requires a CPU helper with a supported symbol: `{}`",
                function.name
            ));
        }
        let body = function
            .body_nodes
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if body.is_empty() || body.len() > 4096 || function.parameters.len() > MAX_SCALAR_SLOTS {
            return Err("native scalar bridge exceeds its function/argument bounds".to_owned());
        }
        let lane = format!("fn:{}", function.name);
        if self.module.nodes.iter().any(|node| {
            self.module.node_lanes.get(&node.name) == Some(&lane)
                && !body.contains(node.name.as_str())
        }) {
            return Err("native scalar bridge lane contains undeclared function nodes".to_owned());
        }
        let mut returns = Vec::new();
        for name in &function.body_nodes {
            let node = self.nodes[name.as_str()];
            let resource = self.resources[node.resource.as_str()];
            if self.module.node_lanes.get(name) != Some(&lane)
                || node.op.module != "cpu"
                || !resource.kind.is_family("cpu")
                || (!admitted(&node.op.instruction)
                    && !self
                        .literal_prints
                        .map(|policy| policy.permit(node, &self.nodes))
                        .transpose()?
                        .unwrap_or(false))
            {
                return Err(format!(
                    "native scalar bridge does not admit {} `{name}`",
                    node.op.full_name()
                ));
            }
            super::loops::validate(node, &self.nodes)?;
            if node.op.instruction.starts_with("return_") {
                returns.push(node);
            }
            let semantics = self
                .registry
                .lookup("cpu")
                .unwrap()
                .describe(node, resource)?;
            if semantics
                .dependencies
                .iter()
                .any(|dependency| !body.contains(dependency.as_str()))
                || self
                    .incoming
                    .get(name.as_str())
                    .is_some_and(|sources| sources.iter().any(|source| !body.contains(source)))
            {
                return Err(format!(
                    "native scalar bridge `{}` requires external initialization or cross-function values",
                    function.name
                ));
            }
            if node.op.instruction.starts_with("param_") {
                let record = yir_domain_cpu::value_parameters::parse(node)?;
                let index = node
                    .op
                    .args
                    .first()
                    .and_then(|arg| arg.parse::<usize>().ok());
                let parameter = index.and_then(|index| function.parameters.get(index));
                if !parameter.is_some_and(|p| {
                    p.node == *name
                        && record.as_ref().map_or_else(
                            || node.op.instruction == format!("param_{}", p.ty),
                            |record| !callback && p.ty == record.layout.type_name,
                        )
                }) {
                    return Err("native scalar bridge parameter node/signature drift".to_owned());
                }
            }
        }
        for (index, parameter) in function.parameters.iter().enumerate() {
            let node = self.nodes[parameter.node.as_str()];
            let record = yir_domain_cpu::value_parameters::parse(node)?;
            let kind_matches = if let Some(record) = record {
                !callback && record.layout.type_name == parameter.ty
            } else {
                ScalarKind::parse(&parameter.ty)?;
                node.op.instruction == format!("param_{}", parameter.ty)
            };
            if !kind_matches
                || node
                    .op
                    .args
                    .first()
                    .and_then(|arg| arg.parse::<usize>().ok())
                    != Some(index)
                || parameter.ownership != YirValueOwnership::Value
            {
                return Err("native scalar bridge parameter node/signature drift".to_owned());
            }
        }
        let result = function
            .result
            .as_ref()
            .ok_or("native scalar helper result missing")?;
        let aggregate = callback || result.ownership == YirValueOwnership::Owned;
        if aggregate && !callback {
            super::aggregates::result_layout(function, &self.nodes)?;
        }
        let expected = if aggregate {
            "return_owned_struct".to_owned()
        } else {
            ScalarKind::parse(&result.ty)?;
            format!("return_{}", result.ty)
        };
        let ownership = if aggregate {
            YirValueOwnership::Owned
        } else {
            YirValueOwnership::Value
        };
        if returns.len() != 1
            || returns[0].name != result.node
            || returns[0].op.instruction != expected
            || result.ownership != ownership
        {
            return Err("native scalar bridge requires one declared typed return".to_owned());
        }
        Ok(())
    }
}

fn symbol_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'$'))
}
