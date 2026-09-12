use super::{CallbackExport, ScalarKind, ScalarStateLayout, MAX_SCALAR_SLOTS};
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{ApplicationSessionSignature, YirModule, YirValueOwnership};

pub(super) fn select(
    module: &YirModule,
    id: &str,
) -> Result<(YirModule, Vec<CallbackExport>, ScalarStateLayout), String> {
    let registry = yir_verify::default_registry();
    yir_verify::verify_module_with_registry(module, &registry)?;
    let registration = yir_core::registered_application_session(module, id)?;
    let signature = ApplicationSessionSignature::bind(module, registration.entries())?;
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.as_str(), resource))
        .collect::<BTreeMap<_, _>>();
    let mut selected_nodes = BTreeSet::new();
    let mut incoming = BTreeMap::<&str, Vec<&str>>::new();
    for edge in &module.edges {
        incoming.entry(&edge.to).or_default().push(&edge.from);
    }
    let mut layouts = Vec::new();
    let mut state_layout = None;
    let mut callbacks = Vec::new();
    let symbol_id = id
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    for (role, function) in [
        ("open", signature.open),
        ("event", signature.event),
        ("close", signature.close),
    ] {
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
        if module.nodes.iter().any(|node| {
            module.node_lanes.get(&node.name) == Some(&lane) && !body.contains(node.name.as_str())
        }) {
            return Err("native scalar bridge lane contains undeclared function nodes".to_owned());
        }
        let mut returns = Vec::new();
        for name in &function.body_nodes {
            let node = nodes[name.as_str()];
            if module.node_lanes.get(name) != Some(&lane)
                || node.op.module != "cpu"
                || !resources[node.resource.as_str()].kind.is_family("cpu")
                || !admitted(&node.op.instruction)
            {
                return Err(format!(
                    "native scalar bridge does not admit {} `{name}`",
                    node.op.full_name()
                ));
            }
            if node.op.instruction == "return_owned_struct" {
                returns.push(node);
            }
            let semantics = registry
                .lookup("cpu")
                .unwrap()
                .describe(node, resources[node.resource.as_str()])?;
            if semantics
                .dependencies
                .iter()
                .any(|dependency| !body.contains(dependency.as_str()))
                || incoming
                    .get(name.as_str())
                    .is_some_and(|sources| sources.iter().any(|source| !body.contains(source)))
            {
                return Err(format!(
                    "native scalar bridge `{}` requires external initialization or calls",
                    function.name
                ));
            }
            if node.op.instruction.starts_with("param_") {
                let index = node
                    .op
                    .args
                    .first()
                    .and_then(|arg| arg.parse::<usize>().ok());
                let parameter = index.and_then(|index| function.parameters.get(index));
                if !parameter.is_some_and(|p| {
                    p.node == *name && node.op.instruction == format!("param_{}", p.ty)
                }) {
                    return Err("native scalar bridge parameter node/signature drift".to_owned());
                }
            }
            selected_nodes.insert(name.clone());
        }
        for parameter in &function.parameters {
            let node = nodes
                .get(parameter.node.as_str())
                .ok_or("native scalar bridge parameter node missing")?;
            if !node.op.instruction.starts_with("param_")
                || parameter.ownership != YirValueOwnership::Value
            {
                return Err(
                    "native scalar bridge parameter must bind a scalar param node".to_owned(),
                );
            }
        }
        let result = function.result.as_ref().unwrap();
        if returns.len() != 1
            || returns[0].name != result.node
            || result.ownership != YirValueOwnership::Owned
        {
            return Err("native scalar bridge requires one declared aggregate return".to_owned());
        }
        let layout = returns[0]
            .op
            .args
            .get(1)
            .ok_or("native scalar bridge return layout missing")?;
        if state_layout.is_none() {
            state_layout = Some(ScalarStateLayout::parse(layout)?);
        }
        let layout = yir_core::parse_owned_struct_layout(layout)?;
        if layout.type_name != result.ty {
            return Err("native scalar bridge result type/layout drift".to_owned());
        }
        layouts.push(layout);
        callbacks.push(CallbackExport {
            role,
            function: function.name.clone(),
            symbol: format!("nuis_native_session_{symbol_id}_{role}_v1"),
            arguments: function
                .parameters
                .iter()
                .map(|p| ScalarKind::parse(&p.ty))
                .collect::<Result<_, _>>()?,
        });
    }
    if layouts.iter().any(|layout| layout != &layouts[0]) {
        return Err("native scalar bridge callback return layouts disagree".to_owned());
    }
    let state_layout = state_layout.unwrap();
    state_layout.bind(module, id)?;
    let mut selected = YirModule::new(module.version.clone());
    selected.functions = [signature.open, signature.event, signature.close]
        .into_iter()
        .cloned()
        .collect();
    selected.application_sessions.push(registration.clone());
    selected.nodes = module
        .nodes
        .iter()
        .filter(|node| selected_nodes.contains(&node.name))
        .cloned()
        .collect();
    selected.edges = module
        .edges
        .iter()
        .filter(|edge| selected_nodes.contains(&edge.from) && selected_nodes.contains(&edge.to))
        .cloned()
        .collect();
    selected.node_lanes = module
        .node_lanes
        .iter()
        .filter(|(name, _)| selected_nodes.contains(*name))
        .map(|(name, lane)| (name.clone(), lane.clone()))
        .collect();
    selected.resources = module
        .resources
        .iter()
        .filter(|r| selected.nodes.iter().any(|n| n.resource == r.name))
        .cloned()
        .collect();
    Ok((selected, callbacks, state_layout))
}

fn symbol_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'$'))
}

// This first native profile has no calls, loops, provider effects or global init.
// Admission is deliberately smaller than general CPU lowering, not a backend dispatch table.
fn admitted(instruction: &str) -> bool {
    matches!(
        instruction,
        "param_bool"
            | "param_i32"
            | "param_i64"
            | "param_f32"
            | "param_f64"
            | "const"
            | "const_bool"
            | "const_i32"
            | "const_i64"
            | "const_f32"
            | "const_f64"
            | "struct"
            | "field"
            | "select"
            | "guard_return"
            | "return_owned_struct"
            | "add"
            | "sub"
            | "mul"
            | "add_i32"
            | "sub_i32"
            | "mul_i32"
            | "add_i64"
            | "sub_i64"
            | "mul_i64"
            | "add_f32"
            | "sub_f32"
            | "mul_f32"
            | "add_f64"
            | "sub_f64"
            | "mul_f64"
            | "eq"
            | "ne"
            | "lt"
            | "le"
            | "gt"
            | "ge"
            | "and"
            | "or"
            | "not"
            | "eq_i64"
            | "ne_i64"
            | "lt_i64"
            | "le_i64"
            | "gt_i64"
            | "ge_i64"
    )
}
