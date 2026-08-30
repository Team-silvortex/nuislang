use std::collections::{HashMap, HashSet};

use yir_core::{Node, YirFunctionRole, YirModule};

pub(crate) fn verify_function_table(
    module: &YirModule,
    nodes: &HashMap<&str, &Node>,
) -> Result<(), String> {
    let mut function_names = HashSet::with_capacity(module.functions.len());
    let mut owned_body_nodes = HashMap::<&str, &str>::with_capacity(module.nodes.len());
    let mut entry_count = 0usize;

    for function in &module.functions {
        if !valid_token(&function.name)
            || !valid_token(&function.domain)
            || !function_names.insert(function.name.as_str())
            || function.body_nodes.is_empty()
        {
            return Err(format!(
                "YIR function `{}` has an invalid or duplicate function boundary",
                function.name
            ));
        }
        if function.role == YirFunctionRole::Entry {
            entry_count += 1;
        }

        let mut body = HashSet::with_capacity(function.body_nodes.len());
        for node in &function.body_nodes {
            if !body.insert(node.as_str()) {
                return Err(format!(
                    "YIR function `{}` contains duplicate body node `{node}`",
                    function.name
                ));
            }
            if !nodes.contains_key(node.as_str()) {
                return Err(format!(
                    "YIR function `{}` references unknown body node `{node}`",
                    function.name
                ));
            }
            if let Some(owner) = owned_body_nodes.insert(node, function.name.as_str()) {
                return Err(format!(
                    "YIR body node `{node}` belongs to both `{owner}` and `{}`",
                    function.name
                ));
            }
        }

        let mut parameter_names = HashSet::with_capacity(function.parameters.len());
        for parameter in &function.parameters {
            if !valid_token(&parameter.name)
                || parameter.ty.is_empty()
                || !parameter_names.insert(parameter.name.as_str())
                || !body.contains(parameter.node.as_str())
            {
                return Err(format!(
                    "YIR function `{}` has invalid parameter `{}`",
                    function.name, parameter.name
                ));
            }
        }
        if let Some(result) = &function.result {
            if result.ty.is_empty() || !body.contains(result.node.as_str()) {
                return Err(format!(
                    "YIR function `{}` has an invalid result boundary",
                    function.name
                ));
            }
        }
    }

    if entry_count > 1 {
        return Err(format!(
            "YIR function table has {entry_count} entry functions; expected at most one"
        ));
    }
    Ok(())
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'.' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{
        Operation, Resource, ResourceKind, YirFunction, YirFunctionParameter, YirFunctionResult,
        YirValueOwnership,
    };

    fn module_with_function() -> YirModule {
        let mut module = YirModule::new("0.1");
        module.resources.push(Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        });
        module.nodes.push(Node {
            name: "input".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
        });
        module.functions.push(YirFunction {
            name: "main".to_owned(),
            domain: "cpu".to_owned(),
            role: YirFunctionRole::Entry,
            parameters: vec![YirFunctionParameter {
                name: "value".to_owned(),
                ty: "i64".to_owned(),
                ownership: YirValueOwnership::Value,
                node: "input".to_owned(),
            }],
            result: Some(YirFunctionResult {
                ty: "i64".to_owned(),
                ownership: YirValueOwnership::Value,
                node: "input".to_owned(),
            }),
            body_nodes: vec!["input".to_owned()],
        });
        module
    }

    #[test]
    fn accepts_typed_function_boundary() {
        let module = module_with_function();
        let nodes = module
            .nodes
            .iter()
            .map(|node| (node.name.as_str(), node))
            .collect();
        verify_function_table(&module, &nodes).unwrap();
    }

    #[test]
    fn rejects_unknown_or_multiply_owned_body_nodes() {
        let mut module = module_with_function();
        let mut duplicate = module.functions[0].clone();
        duplicate.name = "helper".to_owned();
        duplicate.role = YirFunctionRole::Helper;
        module.functions.push(duplicate);
        let nodes = module
            .nodes
            .iter()
            .map(|node| (node.name.as_str(), node))
            .collect();
        assert!(verify_function_table(&module, &nodes)
            .unwrap_err()
            .contains("belongs to both"));

        module.functions.truncate(1);
        module.functions[0].body_nodes[0] = "missing".to_owned();
        let nodes = module
            .nodes
            .iter()
            .map(|node| (node.name.as_str(), node))
            .collect();
        assert!(verify_function_table(&module, &nodes)
            .unwrap_err()
            .contains("unknown body node"));
    }
}
