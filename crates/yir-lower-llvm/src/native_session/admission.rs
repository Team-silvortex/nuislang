use super::{calls, function::FunctionAdmission, CallbackExport, ScalarKind, ScalarStateLayout};
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{ApplicationSessionSignature, YirModule};

pub(super) fn select(
    module: &YirModule,
    id: &str,
) -> Result<(YirModule, Vec<CallbackExport>, ScalarStateLayout), String> {
    let registry = yir_verify::default_registry();
    yir_verify::verify_module_with_registry(module, &registry)?;
    let registration = yir_core::registered_application_session(module, id)?;
    let signature = ApplicationSessionSignature::bind(module, registration.entries())?;
    let mut admission = FunctionAdmission {
        module,
        registry: &registry,
        nodes: module
            .nodes
            .iter()
            .map(|node| (node.name.as_str(), node))
            .collect(),
        resources: module
            .resources
            .iter()
            .map(|r| (r.name.as_str(), r))
            .collect(),
        incoming: BTreeMap::new(),
    };
    for edge in &module.edges {
        admission
            .incoming
            .entry(&edge.to)
            .or_default()
            .push(&edge.from);
    }
    let functions = module
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect::<BTreeMap<_, _>>();
    let roots = [signature.open, signature.event, signature.close];
    let root_names = roots
        .iter()
        .map(|f| f.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut pending = root_names.clone();
    let mut selected_functions = BTreeMap::new();
    let mut selected_nodes = BTreeSet::new();
    let mut graph = BTreeMap::new();
    while let Some(name) = pending.pop_first() {
        if selected_functions.contains_key(name) {
            continue;
        }
        if selected_functions.len() == calls::MAX_FUNCTIONS {
            return Err("native scalar bridge exceeds its reachable-function bound".to_owned());
        }
        let function = functions[name];
        admission.validate(function, root_names.contains(name))?;
        selected_nodes.extend(function.body_nodes.iter().cloned());
        if selected_nodes.len() > calls::MAX_TOTAL_NODES {
            return Err("native scalar bridge exceeds its total-node bound".to_owned());
        }
        let mut callees = BTreeSet::new();
        for node in &function.body_nodes {
            if let Some(callee) = calls::target(admission.nodes[node.as_str()], &functions)? {
                callees.insert(callee.name.clone());
                if !selected_functions.contains_key(callee.name.as_str()) {
                    pending.insert(callee.name.as_str());
                }
            }
        }
        graph.insert(name.to_owned(), callees);
        selected_functions.insert(name, function.clone());
    }
    calls::validate_graph(&graph)?;

    let mut layouts = Vec::new();
    let mut state_layout = None;
    let mut callbacks = Vec::new();
    let symbol_id = id
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    for (role, function) in ["open", "event", "close"].into_iter().zip(roots) {
        let result = function.result.as_ref().unwrap();
        let returned = admission.nodes[result.node.as_str()];
        let layout = returned
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
    selected.functions = selected_functions.into_values().collect();
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

// Calls are admitted only through the bounded, acyclic scalar helper closure.
// Counted scalar loops need a separate termination proof; effects remain excluded.
pub(super) fn admitted(instruction: &str) -> bool {
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
            | "return_bool"
            | "return_i32"
            | "return_i64"
            | "return_f32"
            | "return_f64"
            | "call_bool"
            | "call_i32"
            | "call_i64"
            | "call_f32"
            | "call_f64"
            | "loop_while_i64"
            | "loop_while_i64_chain"
            | "loop_while_scalar_chain"
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
