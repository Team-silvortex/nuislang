use super::*;

pub(super) fn lower_bool(value: bool, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "bool");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "const_bool".to_owned(),
            args: vec![value.to_string()],
        },
    });
    name
}

pub(super) fn lower_text(text: &str, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "text");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "text".to_owned(),
            args: vec![text.to_owned()],
        },
    });
    name
}

pub(super) fn lower_int(value: i64, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "int");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "const_i64".to_owned(),
            args: vec![value.to_string()],
        },
    });
    name
}

pub(super) fn lower_f32(value: &str, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "f32");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "const_f32".to_owned(),
            args: vec![value.to_owned()],
        },
    });
    name
}

pub(super) fn lower_f64(value: &str, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "f64");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "const_f64".to_owned(),
            args: vec![value.to_owned()],
        },
    });
    name
}

pub(super) fn lower_cast_i64_to_i32(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_i32", "cast_i64_to_i32")
}

pub(super) fn lower_cast_i32_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_i64", "cast_i32_to_i64")
}

pub(super) fn lower_cast_i64_to_bool(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_bool", "cast_i64_to_bool")
}

pub(super) fn lower_cast_bool_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_i64", "cast_bool_to_i64")
}

pub(super) fn lower_cast_i64_to_f32(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_f32", "cast_i64_to_f32")
}

pub(super) fn lower_cast_f32_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_i64", "cast_f32_to_i64")
}

pub(super) fn lower_cast_i64_to_f64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_f64", "cast_i64_to_f64")
}

pub(super) fn lower_cast_f64_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    lower_cast_expr(value, state, bindings, "cast_i64", "cast_f64_to_i64")
}

pub(super) fn lower_null(state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "null");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "null".to_owned(),
            args: vec![],
        },
    });
    name
}

fn lower_cast_expr(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    node_prefix: &str,
    instruction: &str,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, node_prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}
