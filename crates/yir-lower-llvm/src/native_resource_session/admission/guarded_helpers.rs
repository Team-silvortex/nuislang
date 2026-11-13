use super::*;
use yir_core::{
    native_resource_session::{
        MixedResourceLayout, MAX_RETURN_HELPER_DEPTH, MAX_RETURN_HELPER_FUNCTIONS,
    },
    Node, YirFunction, YirFunctionRole, YirValueOwnership,
};

pub(super) fn reachable<'a>(
    module: &'a YirModule,
    roots: &[&'a YirFunction],
) -> Result<Vec<&'a YirFunction>, String> {
    fn visit<'a>(
        function: &'a YirFunction,
        module: &'a YirModule,
        active: &mut BTreeSet<String>,
        selected: &mut BTreeMap<String, &'a YirFunction>,
        depths: &mut BTreeMap<String, usize>,
    ) -> Result<usize, String> {
        if active.contains(&function.name) {
            return Err("recursive resource return helper".into());
        }
        if let Some(depth) = depths.get(&function.name) {
            return Ok(*depth);
        }
        if active.len() >= MAX_RETURN_HELPER_DEPTH || selected.len() >= MAX_RETURN_HELPER_FUNCTIONS
        {
            return Err("resource return helpers exceed the closure bound".into());
        }
        selected.insert(function.name.clone(), function);
        active.insert(function.name.clone());
        let mut depth = 1;
        for node in module
            .nodes
            .iter()
            .filter(|n| function.body_nodes.contains(&n.name))
        {
            if !matches!(
                node.op.instruction.as_str(),
                "guard_call_owned_struct_return" | "tail_call_owned_struct_return"
            ) {
                continue;
            }
            if node.op.module != "cpu" {
                return Err("invalid guarded resource helper contract".into());
            }
            let call = parse(node)?;
            let helper = module
                .functions
                .iter()
                .find(|f| f.name == call.callee)
                .ok_or("missing guarded resource helper")?;
            if helper.domain != "cpu"
                || helper.role != YirFunctionRole::Helper
                || helper.result.as_ref().map(|r| (&r.ty, &r.ownership))
                    != function.result.as_ref().map(|r| (&r.ty, &r.ownership))
            {
                return Err("guarded resource helper changes its result contract".into());
            }
            depth = depth.max(1 + visit(helper, module, active, selected, depths)?);
            if depth > MAX_RETURN_HELPER_DEPTH {
                return Err("resource return helpers exceed the closure depth bound".into());
            }
        }
        active.remove(&function.name);
        depths.insert(function.name.clone(), depth);
        Ok(depth)
    }
    let mut selected = BTreeMap::new();
    let mut depths = BTreeMap::new();
    for root in roots {
        visit(
            root,
            module,
            &mut BTreeSet::new(),
            &mut selected,
            &mut depths,
        )?;
    }
    Ok(selected.into_values().collect())
}

pub(super) fn check(
    node: &Node,
    module: &YirModule,
    layout: Option<&MixedResourceLayout>,
    values: &BTreeMap<String, Kind>,
    owners: &BTreeSet<String>,
) -> Result<(), String> {
    let layout = layout.ok_or("guarded resource return requires a State layout")?;
    let call = parse(node)?;
    if call.layout != layout.encoded
        || (node.op.instruction != "tail_call_owned_struct_return"
            && values.get(call.condition) != Some(&Kind::Bool))
    {
        return Err("guarded resource return condition or layout drift".into());
    }
    // Both calls partition the complete live set. The caller preserves it for
    // a guard's untaken edge and consumes it after a terminal call is validated.
    if call
        .transfers
        .iter()
        .chain(call.cleanup)
        .cloned()
        .collect::<BTreeSet<_>>()
        != *owners
        || call
            .transfers
            .iter()
            .chain(call.cleanup)
            .any(|n| values.get(n) != Some(&Kind::Bytes))
    {
        return Err("guarded resource ownership must match all live caller owners as Bytes".into());
    }
    let helper = module
        .functions
        .iter()
        .find(|f| f.name == call.callee)
        .ok_or("missing guarded resource helper")?;
    let returned = helper
        .result
        .as_ref()
        .and_then(|r| module.nodes.iter().find(|n| n.name == r.node))
        .ok_or("missing guarded resource helper return")?;
    if !matches!(
        returned.op.instruction.as_str(),
        "return_owned_struct" | "tail_call_owned_struct_return"
    ) || returned.op.args.get(1) != Some(&layout.encoded)
        || helper.parameters.len() != call.inputs.len() + call.transfers.len()
    {
        return Err("guarded resource helper ABI drift".into());
    }
    for (index, (input, param)) in call
        .inputs
        .iter()
        .chain(call.transfers)
        .zip(&helper.parameters)
        .enumerate()
    {
        let kind = if index == 0 {
            Kind::Buffer
        } else if index < call.inputs.len() {
            Kind::I64
        } else {
            Kind::Bytes
        };
        if values.get(input) != Some(&kind) || parameter_kind(param, index)? != kind {
            return Err("guarded resource helper capture drift".into());
        }
    }
    Ok(())
}

fn parse(node: &Node) -> Result<yir_core::guarded_owned_call::GuardedOwnedCall<'_>, String> {
    if node.op.instruction == "tail_call_owned_struct_return" {
        yir_core::guarded_owned_call::parse_tail(&node.op.args)
    } else {
        yir_core::guarded_owned_call::parse(&node.op.args)
    }
}

pub(super) fn parameter_kind(
    param: &yir_core::YirFunctionParameter,
    index: usize,
) -> Result<Kind, String> {
    match (index, param.ty.as_str(), &param.ownership) {
        (0, "ref Buffer", YirValueOwnership::Borrowed) => Ok(Kind::Buffer),
        (1.., "i64", YirValueOwnership::Value) => Ok(Kind::I64),
        (1.., "Bytes", YirValueOwnership::Owned) => Ok(Kind::Bytes),
        _ => Err("resource helper parameter ownership or type drift".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain(count: usize) -> YirModule {
        let mut module = YirModule::new("0.1".to_owned());
        for index in 0..count {
            let name = format!("helper{index}");
            let call = format!("call{index}");
            let body = if index + 1 < count {
                module.nodes.push(Node {
                    name: call.clone(),
                    resource: "cpu0".into(),
                    op: yir_core::Operation::parse(
                        "cpu.guard_call_owned_struct_return",
                        vec![
                            "ready".into(),
                            format!("helper{}", index + 1),
                            "State{pixels:Bytes}".into(),
                            "buffer".into(),
                        ],
                    )
                    .unwrap(),
                });
                vec![call]
            } else {
                vec![]
            };
            module.functions.push(YirFunction {
                name,
                domain: "cpu".into(),
                role: YirFunctionRole::Helper,
                parameters: vec![],
                result: None,
                body_nodes: body,
            });
        }
        module
    }

    #[test]
    fn cached_helper_depth_cannot_bypass_resource_closure_budget() {
        for count in [8, 9] {
            for tail in [false, true] {
                let mut module = chain(count);
                if tail {
                    for node in &mut module.nodes {
                        node.op.instruction = "tail_call_owned_struct_return".into();
                        node.op.args.remove(0);
                    }
                }
                // Discover the shared suffix before its longer caller path.
                let roots = [&module.functions[count - 2], &module.functions[0]];
                assert_eq!(reachable(&module, &roots).is_ok(), count == 8);
            }
        }
        let mut cycle = chain(3);
        cycle.nodes[1].op.args[1] = "helper0".into();
        assert!(reachable(&cycle, &[&cycle.functions[0]])
            .unwrap_err()
            .contains("recursive"));
    }
}
