use std::collections::BTreeMap;

use yir_core::{EdgeKind, Resource, YirModule};
use yir_verify::verify_module;

#[derive(Clone)]
enum LlvmValueRef {
    I64(String),
    Ptr(String),
    Void,
}

pub fn emit_module(module: &YirModule) -> Result<String, String> {
    verify_module(module)?;

    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<String, &Resource>>();

    let mut body = Vec::new();
    let mut registers = BTreeMap::<String, LlvmValueRef>::new();
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
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "null") => {
                registers.insert(node.name.clone(), LlvmValueRef::Ptr("null".to_owned()));
            }
            ("cpu", "borrow") | ("cpu", "move_ptr") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for {} `{}` because its input is outside the current CPU LLVM slice",
                        node.op.full_name(),
                        node.name
                    ));
                    continue;
                };
                registers.insert(node.name.clone(), LlvmValueRef::Ptr(ptr.to_owned()));
            }
            ("cpu", "add") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.add `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "sub") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.sub `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sub i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "mul") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.mul `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = mul i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "madd") => {
                let (Some(lhs), Some(rhs), Some(acc)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                    get_i64(&registers, &node.op.args[2]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.madd `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let mul = fresh_reg(&mut next_reg);
                body.push(format!("  {mul} = mul i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 {mul}, {acc}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "alloc_node") => {
                let (Some(value), Some(next_ptr)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_ptr(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.alloc_node `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let raw = fresh_reg(&mut next_reg);
                body.push(format!("  {raw} = call ptr @malloc(i64 16)"));
                let value_slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {value_slot} = getelementptr inbounds %cpu.node, ptr {raw}, i32 0, i32 0"
                ));
                body.push(format!("  store i64 {value}, ptr {value_slot}"));
                let next_slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_slot} = getelementptr inbounds %cpu.node, ptr {raw}, i32 0, i32 1"
                ));
                body.push(format!("  store ptr {next_ptr}, ptr {next_slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Ptr(raw));
            }
            ("cpu", "load_value") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.load_value `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 0"
                ));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = load i64, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "load_next") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.load_next `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 1"
                ));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = load ptr, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Ptr(reg));
            }
            ("cpu", "store_value") => {
                let (Some(ptr), Some(value)) = (
                    get_ptr(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.store_value `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 0"
                ));
                body.push(format!("  store i64 {value}, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Void);
            }
            ("cpu", "store_next") => {
                let (Some(ptr), Some(next_ptr)) = (
                    get_ptr(&registers, &node.op.args[0]),
                    get_ptr(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.store_next `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 1"
                ));
                body.push(format!("  store ptr {next_ptr}, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Void);
            }
            ("cpu", "is_null") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.is_null `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp eq ptr {ptr}, null"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "free") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.free `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                body.push(format!("  call void @free(ptr {ptr})"));
                registers.insert(node.name.clone(), LlvmValueRef::Void);
            }
            ("cpu", "input_i64") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  ; static AOT lowering freezes cpu.input_i64 `{}` to its default value",
                    node.op.args[0]
                ));
                body.push(format!("  {reg} = add i64 0, {}", node.op.args[1]));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "print") => {
                if let Some(input) = get_i64(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                    last_cpu_value = Some(input.to_owned());
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.print `{}` because its input is produced outside the current CPU LLVM slice",
                        node.op.args[0]
                    ));
                }
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
%cpu.node = type {{ i64, ptr }}\n\
declare ptr @malloc(i64)\n\
declare void @free(ptr)\n\
declare void @nuis_debug_print_i64(i64)\n\n\
define i64 @nuis_yir_entry() {{\n{}\n  ret i64 {}\n}}\n",
        module.version,
        body.join("\n"),
        ret
    ))
}

fn get_i64<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::I64(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_ptr<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::Ptr(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn fresh_reg(next: &mut usize) -> String {
    *next += 1;
    let reg = format!("%{}", *next);
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
