use nuis_semantics::model::NirTypeRef;

use super::*;

pub(super) fn lower_core_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::Bool(value) => Some(Ok(lower_bool(*value, state))),
        NirExpr::Text(text) => Some(Ok(lower_text(text, state))),
        NirExpr::Int(value) => Some(Ok(lower_int(*value, state))),
        NirExpr::F32(value) => Some(Ok(lower_f32(value, state))),
        NirExpr::F64(value) => Some(Ok(lower_f64(value, state))),
        NirExpr::CastI64ToI32(value) => Some(lower_cast_i64_to_i32(value, state, bindings)),
        NirExpr::CastI32ToI64(value) => Some(lower_cast_i32_to_i64(value, state, bindings)),
        NirExpr::CastI64ToBool(value) => Some(lower_cast_i64_to_bool(value, state, bindings)),
        NirExpr::CastBoolToI64(value) => Some(lower_cast_bool_to_i64(value, state, bindings)),
        NirExpr::CastI64ToF32(value) => Some(lower_cast_i64_to_f32(value, state, bindings)),
        NirExpr::CastF32ToI64(value) => Some(lower_cast_f32_to_i64(value, state, bindings)),
        NirExpr::CastI64ToF64(value) => Some(lower_cast_i64_to_f64(value, state, bindings)),
        NirExpr::CastF64ToI64(value) => Some(lower_cast_f64_to_i64(value, state, bindings)),
        NirExpr::Var(name) => Some(
            bindings
                .get(name)
                .cloned()
                .ok_or_else(|| format!("minimal nuisc lowering found unbound variable `{name}`")),
        ),
        NirExpr::Null => Some(Ok(lower_null(state))),
        NirExpr::Borrow(value) => Some(lower_unary_cpu_expr("borrow", value, state, bindings)),
        NirExpr::BorrowEnd(value) => {
            Some(lower_unary_cpu_expr("borrow_end", value, state, bindings))
        }
        NirExpr::HostBufferHandle(value) => Some(lower_expr(value, state, bindings)),
        NirExpr::Move(value) => Some(lower_move(value, state, bindings)),
        NirExpr::AllocNode { value, next } => Some(lower_alloc_node(value, next, state, bindings)),
        NirExpr::AllocBuffer { len, fill } => Some(lower_alloc_buffer(len, fill, state, bindings)),
        NirExpr::LoadValue(value) => {
            Some(lower_unary_cpu_expr("load_value", value, state, bindings))
        }
        NirExpr::LoadNext(value) => Some(lower_unary_cpu_expr("load_next", value, state, bindings)),
        NirExpr::BufferLen(value) => {
            Some(lower_unary_cpu_expr("buffer_len", value, state, bindings))
        }
        NirExpr::IsNull(value) => Some(lower_unary_cpu_expr("is_null", value, state, bindings)),
        NirExpr::LoadAt { buffer, index } => Some(lower_load_at(buffer, index, state, bindings)),
        NirExpr::StoreValue { target, value } => {
            Some(lower_store_value(target, value, state, bindings))
        }
        NirExpr::StoreNext { target, next } => {
            Some(lower_store_next(target, next, state, bindings))
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => Some(lower_store_at(buffer, index, value, state, bindings)),
        NirExpr::Free(value) => Some(lower_free(value, state, bindings)),
        NirExpr::Binary { op, lhs, rhs } => Some(lower_binary(op, lhs, rhs, state, bindings)),
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => Some(lower_struct_literal(
            type_name, type_args, fields, state, bindings,
        )),
        NirExpr::FieldAccess { base, field } => {
            Some(lower_field_access(base, field, state, bindings))
        }
        _ => None,
    }
}

fn lower_bool(value: bool, state: &mut LoweringState<'_>) -> String {
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

fn lower_text(text: &str, state: &mut LoweringState<'_>) -> String {
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

fn lower_int(value: i64, state: &mut LoweringState<'_>) -> String {
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

fn lower_f32(value: &str, state: &mut LoweringState<'_>) -> String {
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

fn lower_f64(value: &str, state: &mut LoweringState<'_>) -> String {
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

fn lower_cast_i64_to_i32(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_i32");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_i64_to_i32".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_cast_i32_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_i64");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_i32_to_i64".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_cast_i64_to_bool(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_bool");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_i64_to_bool".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_cast_bool_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_i64");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_bool_to_i64".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_cast_i64_to_f32(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_f32");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_i64_to_f32".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_cast_f32_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_i64");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_f32_to_i64".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_cast_i64_to_f64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_f64");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_i64_to_f64".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_cast_f64_to_i64(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let input = lower_expr(value, state, bindings)?;
    let name = next_name(state, "cast_i64");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "cast_f64_to_i64".to_owned(),
            args: vec![input.clone()],
        },
    });
    push_dep_edges(state, &input, &name);
    Ok(name)
}

fn lower_null(state: &mut LoweringState<'_>) -> String {
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

fn lower_move(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let ptr = lower_expr(value, state, bindings)?;
    let name = next_name(state, "move");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "move_ptr".to_owned(),
            args: vec![ptr.clone()],
        },
    });
    push_dep_edges(state, &ptr, &name);
    push_lifetime_edge(state, &ptr, &name);
    Ok(name)
}

fn lower_alloc_node(
    value: &NirExpr,
    next: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let value_name = lower_expr(value, state, bindings)?;
    let next_ptr_name = lower_expr(next, state, bindings)?;
    let name = next_name(state, "alloc_node");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "alloc_node".to_owned(),
            args: vec![value_name.clone(), next_ptr_name.clone()],
        },
    });
    push_dep_edges(state, &value_name, &name);
    push_dep_edges(state, &next_ptr_name, &name);
    Ok(name)
}

fn lower_alloc_buffer(
    len: &NirExpr,
    fill: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let len_name = lower_expr(len, state, bindings)?;
    let fill_name = lower_expr(fill, state, bindings)?;
    let name = next_name(state, "alloc_buffer");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "alloc_buffer".to_owned(),
            args: vec![len_name.clone(), fill_name.clone()],
        },
    });
    push_dep_edges(state, &len_name, &name);
    push_dep_edges(state, &fill_name, &name);
    Ok(name)
}

fn lower_load_at(
    buffer: &NirExpr,
    index: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let buffer_name = lower_expr(buffer, state, bindings)?;
    let index_name = lower_expr(index, state, bindings)?;
    let name = next_name(state, "load_at");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "load_at".to_owned(),
            args: vec![buffer_name.clone(), index_name.clone()],
        },
    });
    push_dep_edges(state, &buffer_name, &name);
    push_dep_edges(state, &index_name, &name);
    Ok(name)
}

fn lower_store_value(
    target: &NirExpr,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let target_name = lower_expr(target, state, bindings)?;
    let value_name = lower_expr(value, state, bindings)?;
    let name = next_name(state, "store_value");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "store_value".to_owned(),
            args: vec![target_name.clone(), value_name.clone()],
        },
    });
    push_dep_edges(state, &target_name, &name);
    push_dep_edges(state, &value_name, &name);
    push_lifetime_edge(state, &target_name, &name);
    Ok(name)
}

fn lower_store_next(
    target: &NirExpr,
    next: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let target_name = lower_expr(target, state, bindings)?;
    let next_name_value = lower_expr(next, state, bindings)?;
    let name = next_name(state, "store_next");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "store_next".to_owned(),
            args: vec![target_name.clone(), next_name_value.clone()],
        },
    });
    push_dep_edges(state, &target_name, &name);
    push_dep_edges(state, &next_name_value, &name);
    push_lifetime_edge(state, &target_name, &name);
    Ok(name)
}

fn lower_store_at(
    buffer: &NirExpr,
    index: &NirExpr,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let buffer_name = lower_expr(buffer, state, bindings)?;
    let index_name = lower_expr(index, state, bindings)?;
    let value_name = lower_expr(value, state, bindings)?;
    let name = next_name(state, "store_at");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "store_at".to_owned(),
            args: vec![buffer_name.clone(), index_name.clone(), value_name.clone()],
        },
    });
    push_dep_edges(state, &buffer_name, &name);
    push_dep_edges(state, &index_name, &name);
    push_dep_edges(state, &value_name, &name);
    push_lifetime_edge(state, &buffer_name, &name);
    Ok(name)
}

fn lower_free(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let ptr = lower_expr(value, state, bindings)?;
    let name = next_name(state, "free");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "free".to_owned(),
            args: vec![ptr.clone()],
        },
    });
    push_dep_edges(state, &ptr, &name);
    push_lifetime_edge(state, &ptr, &name);
    Ok(name)
}

fn lower_binary(
    op: &NirBinaryOp,
    lhs: &NirExpr,
    rhs: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let lhs_name = lower_expr(lhs, state, bindings)?;
    let rhs_name = lower_expr(rhs, state, bindings)?;
    let instruction = match op {
        NirBinaryOp::And => "and",
        NirBinaryOp::Or => "or",
        NirBinaryOp::Add => "add",
        NirBinaryOp::Sub => "sub",
        NirBinaryOp::Mul => "mul",
        NirBinaryOp::Div => "div",
        NirBinaryOp::Rem => "rem",
        NirBinaryOp::Eq => "eq",
        NirBinaryOp::Ne => "ne",
        NirBinaryOp::Lt => "lt",
        NirBinaryOp::Le => "le",
        NirBinaryOp::Gt => "gt",
        NirBinaryOp::Ge => "ge",
    };
    let name = next_name(state, instruction);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![lhs_name.clone(), rhs_name.clone()],
        },
    });
    push_dep_edges(state, &lhs_name, &name);
    push_dep_edges(state, &rhs_name, &name);
    Ok(name)
}

fn lower_struct_literal(
    type_name: &str,
    _type_args: &[NirTypeRef],
    fields: &[(String, NirExpr)],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut args_out = vec![type_name.to_owned()];
    let name = next_name(state, "struct");
    let mut lowered_fields = Vec::new();
    for (field_name, field_expr) in fields {
        let lowered = lower_expr(field_expr, state, bindings)?;
        lowered_fields.push(lowered.clone());
        args_out.push(format!("{field_name}={lowered}"));
    }
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "struct".to_owned(),
            args: args_out,
        },
    });
    for lowered in lowered_fields {
        push_dep_edges(state, &lowered, &name);
    }
    Ok(name)
}

fn lower_field_access(
    base: &NirExpr,
    field: &str,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let base_name = lower_expr(base, state, bindings)?;
    let name = next_name(state, "field");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "field".to_owned(),
            args: vec![base_name.clone(), field.to_owned()],
        },
    });
    push_dep_edges(state, &base_name, &name);
    Ok(name)
}
