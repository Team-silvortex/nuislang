use std::collections::BTreeMap;

use yir_core::{EdgeKind, Resource, YirModule};
use yir_verify::verify_module;

pub fn emit_module(module: &YirModule) -> Result<String, String> {
    verify_module(module)?;

    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<String, &Resource>>();

    let mut body = Vec::new();
    let mut registers = BTreeMap::<String, String>::new();
    let mut next_reg = 0usize;
    let mut last_cpu_value = None::<String>;

    for node_name in topological_order(module)? {
        let node = module
            .nodes
            .iter()
            .find(|node| node.name == node_name)
            .ok_or_else(|| format!("lowering references unknown node `{node_name}`"))?;
        let resource = resources
            .get(&node.resource)
            .copied()
            .ok_or_else(|| format!("unknown resource `{}`", node.resource))?;

        if !resource.kind.is_family("cpu") {
            body.push(format!(
                "  ; deferred lowering for {} on {} ({})",
                node.op.full_name(),
                node.resource,
                resource.kind.raw
            ));
            continue;
        }

        match (node.op.module.as_str(), node.op.instruction.as_str()) {
            ("cpu", "const") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 0, {}", node.op.args[0]));
                registers.insert(node.name.clone(), reg.clone());
                last_cpu_value = Some(reg);
            }
            ("cpu", "add") => {
                let lhs = registers
                    .get(&node.op.args[0])
                    .ok_or_else(|| format!("missing llvm input for `{}`", node.op.args[0]))?;
                let rhs = registers
                    .get(&node.op.args[1])
                    .ok_or_else(|| format!("missing llvm input for `{}`", node.op.args[1]))?;
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), reg.clone());
                last_cpu_value = Some(reg);
            }
            ("cpu", "mul") => {
                let lhs = registers
                    .get(&node.op.args[0])
                    .ok_or_else(|| format!("missing llvm input for `{}`", node.op.args[0]))?;
                let rhs = registers
                    .get(&node.op.args[1])
                    .ok_or_else(|| format!("missing llvm input for `{}`", node.op.args[1]))?;
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = mul i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), reg.clone());
                last_cpu_value = Some(reg);
            }
            ("cpu", "print") => {
                let input = registers
                    .get(&node.op.args[0])
                    .ok_or_else(|| format!("missing llvm input for `{}`", node.op.args[0]))?;
                body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                last_cpu_value = Some(input.clone());
            }
            _ => {
                body.push(format!(
                    "  ; deferred lowering for {} on {} ({})",
                    node.op.full_name(),
                    node.resource,
                    resource.kind.raw
                ));
            }
        }
    }

    let ret = last_cpu_value.unwrap_or_else(|| "0".to_owned());

    Ok(format!(
        "; yir version: {}\n\
declare void @nuis_debug_print_i64(i64)\n\n\
define i64 @nuis_yir_entry() {{\n{}\n  ret i64 {}\n}}\n",
        module.version,
        body.join("\n"),
        ret
    ))
}

fn fresh_reg(next: &mut usize) -> String {
    let reg = format!("%{}", *next);
    *next += 1;
    reg
}

fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();

    for node in &module.nodes {
        adjacency.entry(node.name.clone()).or_default();
        indegree.entry(node.name.clone()).or_insert(0);
    }

    for edge in &module.edges {
        match edge.kind {
            EdgeKind::Dep | EdgeKind::Effect | EdgeKind::Lifetime | EdgeKind::CrossDomainExchange => {
                adjacency
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
                *indegree.entry(edge.to.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
        .collect::<Vec<_>>();
    ready.sort();

    let mut order = Vec::with_capacity(module.nodes.len());
    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target.clone());
                        ready.sort();
                    }
                }
            }
        }
    }

    if order.len() != module.nodes.len() {
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(order)
}
