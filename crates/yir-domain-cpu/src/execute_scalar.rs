use yir_core::{ExecutionState, Node, Value};

use crate::runtime_helpers::select_variant_union;

pub(crate) fn execute_cpu_scalar_node(
    node: &Node,
    state: &ExecutionState,
) -> Result<Option<Value>, String> {
    let value = match node.op.instruction.as_str() {
        "neg" => Ok(Value::Int(-state.expect_int(&node.op.args[0])?)),
        "not" => Ok(Value::Int(!state.expect_int(&node.op.args[0])?)),
        "add" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int(lhs + rhs))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::F32(lhs + rhs))
            } else {
                Ok(Value::F64(
                    state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
                ))
            }
        }
        "add_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? + state.expect_i32(&node.op.args[1])?,
        )),
        "add_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? + state.expect_f32(&node.op.args[1])?,
        )),
        "add_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
        )),
        "sub" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int(lhs - rhs))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::F32(lhs - rhs))
            } else {
                Ok(Value::F64(
                    state.expect_f64(&node.op.args[0])? - state.expect_f64(&node.op.args[1])?,
                ))
            }
        }
        "sub_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? - state.expect_i32(&node.op.args[1])?,
        )),
        "sub_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? - state.expect_f32(&node.op.args[1])?,
        )),
        "sub_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? - state.expect_f64(&node.op.args[1])?,
        )),
        "mul" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int(lhs * rhs))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::F32(lhs * rhs))
            } else {
                Ok(Value::F64(
                    state.expect_f64(&node.op.args[0])? * state.expect_f64(&node.op.args[1])?,
                ))
            }
        }
        "mul_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? * state.expect_i32(&node.op.args[1])?,
        )),
        "mul_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? * state.expect_f32(&node.op.args[1])?,
        )),
        "mul_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? * state.expect_f64(&node.op.args[1])?,
        )),
        "div" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                if rhs == 0 {
                    return Err(format!("node `{}` divides by zero", node.name));
                }
                Ok(Value::Int(lhs / rhs))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                if rhs == 0.0 {
                    return Err(format!("node `{}` divides by zero", node.name));
                }
                Ok(Value::F32(lhs / rhs))
            } else {
                let lhs = state.expect_f64(&node.op.args[0])?;
                let rhs = state.expect_f64(&node.op.args[1])?;
                if rhs == 0.0 {
                    return Err(format!("node `{}` divides by zero", node.name));
                }
                Ok(Value::F64(lhs / rhs))
            }
        }
        "div_i32" => {
            let lhs = state.expect_i32(&node.op.args[0])?;
            let rhs = state.expect_i32(&node.op.args[1])?;
            if rhs == 0 {
                return Err(format!("node `{}` divides by zero", node.name));
            }
            Ok(Value::I32(lhs / rhs))
        }
        "div_f32" => {
            let lhs = state.expect_f32(&node.op.args[0])?;
            let rhs = state.expect_f32(&node.op.args[1])?;
            if rhs == 0.0 {
                return Err(format!("node `{}` divides by zero", node.name));
            }
            Ok(Value::F32(lhs / rhs))
        }
        "div_f64" => {
            let lhs = state.expect_f64(&node.op.args[0])?;
            let rhs = state.expect_f64(&node.op.args[1])?;
            if rhs == 0.0 {
                return Err(format!("node `{}` divides by zero", node.name));
            }
            Ok(Value::F64(lhs / rhs))
        }
        "rem" => {
            let lhs = state.expect_int(&node.op.args[0])?;
            let rhs = state.expect_int(&node.op.args[1])?;
            if rhs == 0 {
                return Err(format!("node `{}` computes remainder by zero", node.name));
            }
            Ok(Value::Int(lhs % rhs))
        }
        "eq" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs == rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_i32(&node.op.args[0]),
                state.expect_i32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs == rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs == rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f64(&node.op.args[0]),
                state.expect_f64(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs == rhs) as i64))
            } else {
                Ok(Value::Int(
                    (state.expect_bool(&node.op.args[0])? == state.expect_bool(&node.op.args[1])?)
                        as i64,
                ))
            }
        }
        "eq_i32" => Ok(Value::Bool(
            state.expect_i32(&node.op.args[0])? == state.expect_i32(&node.op.args[1])?,
        )),
        "eq_f32" => Ok(Value::Bool(
            state.expect_f32(&node.op.args[0])? == state.expect_f32(&node.op.args[1])?,
        )),
        "eq_f64" => Ok(Value::Bool(
            state.expect_f64(&node.op.args[0])? == state.expect_f64(&node.op.args[1])?,
        )),
        "ne" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs != rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_i32(&node.op.args[0]),
                state.expect_i32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs != rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs != rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f64(&node.op.args[0]),
                state.expect_f64(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs != rhs) as i64))
            } else {
                Ok(Value::Int(
                    (state.expect_bool(&node.op.args[0])? != state.expect_bool(&node.op.args[1])?)
                        as i64,
                ))
            }
        }
        "lt" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs < rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs < rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f64(&node.op.args[0]),
                state.expect_f64(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs < rhs) as i64))
            } else {
                Ok(Value::Int(
                    (state.expect_i32(&node.op.args[0])? < state.expect_i32(&node.op.args[1])?)
                        as i64,
                ))
            }
        }
        "lt_i32" => Ok(Value::Bool(
            state.expect_i32(&node.op.args[0])? < state.expect_i32(&node.op.args[1])?,
        )),
        "lt_f32" => Ok(Value::Bool(
            state.expect_f32(&node.op.args[0])? < state.expect_f32(&node.op.args[1])?,
        )),
        "lt_f64" => Ok(Value::Bool(
            state.expect_f64(&node.op.args[0])? < state.expect_f64(&node.op.args[1])?,
        )),
        "gt" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs > rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs > rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f64(&node.op.args[0]),
                state.expect_f64(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs > rhs) as i64))
            } else {
                Ok(Value::Int(
                    (state.expect_i32(&node.op.args[0])? > state.expect_i32(&node.op.args[1])?)
                        as i64,
                ))
            }
        }
        "gt_i32" => Ok(Value::Bool(
            state.expect_i32(&node.op.args[0])? > state.expect_i32(&node.op.args[1])?,
        )),
        "gt_f32" => Ok(Value::Bool(
            state.expect_f32(&node.op.args[0])? > state.expect_f32(&node.op.args[1])?,
        )),
        "gt_f64" => Ok(Value::Bool(
            state.expect_f64(&node.op.args[0])? > state.expect_f64(&node.op.args[1])?,
        )),
        "le" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs <= rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs <= rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f64(&node.op.args[0]),
                state.expect_f64(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs <= rhs) as i64))
            } else {
                Ok(Value::Int(
                    (state.expect_i32(&node.op.args[0])? <= state.expect_i32(&node.op.args[1])?)
                        as i64,
                ))
            }
        }
        "ge" => {
            if let (Ok(lhs), Ok(rhs)) = (
                state.expect_int(&node.op.args[0]),
                state.expect_int(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs >= rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f32(&node.op.args[0]),
                state.expect_f32(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs >= rhs) as i64))
            } else if let (Ok(lhs), Ok(rhs)) = (
                state.expect_f64(&node.op.args[0]),
                state.expect_f64(&node.op.args[1]),
            ) {
                Ok(Value::Int((lhs >= rhs) as i64))
            } else {
                Ok(Value::Int(
                    (state.expect_i32(&node.op.args[0])? >= state.expect_i32(&node.op.args[1])?)
                        as i64,
                ))
            }
        }
        "and" => Ok(Value::Int(
            state.expect_int(&node.op.args[0])? & state.expect_int(&node.op.args[1])?,
        )),
        "or" => Ok(Value::Int(
            state.expect_int(&node.op.args[0])? | state.expect_int(&node.op.args[1])?,
        )),
        "xor" => Ok(Value::Int(
            state.expect_int(&node.op.args[0])? ^ state.expect_int(&node.op.args[1])?,
        )),
        "shl" => {
            let lhs = state.expect_int(&node.op.args[0])?;
            let rhs = state.expect_int(&node.op.args[1])?;
            if rhs < 0 {
                return Err(format!("node `{}` shifts by negative amount", node.name));
            }
            Ok(Value::Int(lhs.wrapping_shl(rhs as u32)))
        }
        "shr" => {
            let lhs = state.expect_int(&node.op.args[0])?;
            let rhs = state.expect_int(&node.op.args[1])?;
            if rhs < 0 {
                return Err(format!("node `{}` shifts by negative amount", node.name));
            }
            Ok(Value::Int(lhs >> rhs))
        }
        "madd" => Ok(Value::Int(
            state.expect_int(&node.op.args[0])? * state.expect_int(&node.op.args[1])?
                + state.expect_int(&node.op.args[2])?,
        )),
        "select" | "select_owned_bytes" | "select_owned_bytes_drop_unselected" => {
            let cond = match state.expect_value(&node.op.args[0])? {
                Value::Bool(value) => *value,
                Value::Int(value) => *value != 0,
                other => {
                    return Err(format!(
                        "node `{}` expects bool or i64 select condition, got {}",
                        node.name, other
                    ))
                }
            };
            let then_value = state.expect_value(&node.op.args[1])?;
            let else_value = state.expect_value(&node.op.args[2])?;
            if node.op.instruction != "select"
                && (!matches!(then_value, Value::OwnedBytes(_))
                    || !matches!(else_value, Value::OwnedBytes(_)))
            {
                return Err(format!(
                    "node `{}` expects owned bytes in both select branches",
                    node.name
                ));
            }
            if let Some(union) = select_variant_union(cond, then_value, else_value) {
                return Ok(Some(Value::VariantUnion(union)));
            }
            Ok(if cond {
                then_value.clone()
            } else {
                else_value.clone()
            })
        }
        "select_owned_bytes_tree" => {
            let args = yir_core::parse_owned_select_tree_args(&node.op.args).ok_or_else(|| {
                format!(
                    "node `{}` has invalid owned select tree arguments",
                    node.name
                )
            })?;
            let selected = select_owned_tree_leaf(&args.tree, state, &node.name)?;
            let owner_index = match selected {
                yir_core::OwnedSelectTree::Owner(index) => *index,
                yir_core::OwnedSelectTree::Call {
                    scalar_args, owner, ..
                } => {
                    for arg in *scalar_args {
                        state.expect_value(arg)?;
                    }
                    *owner
                }
                yir_core::OwnedSelectTree::If { .. } => unreachable!(),
            };
            let owner = args.owners.get(owner_index).ok_or_else(|| {
                format!(
                    "node `{}` selects unknown owner index {owner_index}",
                    node.name
                )
            })?;
            let Value::OwnedBytes(bytes) = state.expect_value(owner)? else {
                return Err(format!(
                    "node `{}` expects owned bytes for tree owner `{owner}`",
                    node.name
                ));
            };
            Ok(Value::OwnedBytes(bytes.clone()))
        }
        "cast_bool_to_i64" => Ok(Value::Int(if state.expect_bool(&node.op.args[0])? {
            1
        } else {
            0
        })),
        "cast_i32_to_i64" => Ok(Value::Int(state.expect_i32(&node.op.args[0])? as i64)),
        "cast_i64_to_bool" => Ok(Value::Bool(state.expect_int(&node.op.args[0])? != 0)),
        "cast_i64_to_i32" => Ok(Value::I32(state.expect_int(&node.op.args[0])? as i32)),
        "cast_i32_to_f32" => Ok(Value::F32(state.expect_i32(&node.op.args[0])? as f32)),
        "cast_i32_to_f64" => Ok(Value::F64(state.expect_i32(&node.op.args[0])? as f64)),
        "cast_f32_to_f64" => Ok(Value::F64(state.expect_f32(&node.op.args[0])? as f64)),
        "cast_f64_to_f32" => Ok(Value::F32(state.expect_f64(&node.op.args[0])? as f32)),

        _ => return Ok(None),
    };
    value.map(Some)
}

fn select_owned_tree_leaf<'a>(
    tree: &'a yir_core::OwnedSelectTree<'a>,
    state: &ExecutionState,
    node_name: &str,
) -> Result<&'a yir_core::OwnedSelectTree<'a>, String> {
    match tree {
        yir_core::OwnedSelectTree::Owner(_) | yir_core::OwnedSelectTree::Call { .. } => Ok(tree),
        yir_core::OwnedSelectTree::If {
            condition,
            then_tree,
            else_tree,
        } => {
            let condition = match state.expect_value(condition)? {
                Value::Bool(value) => *value,
                Value::Int(value) => *value != 0,
                other => {
                    return Err(format!(
                        "node `{node_name}` expects bool or i64 tree condition, got {other}"
                    ))
                }
            };
            select_owned_tree_leaf(
                if condition { then_tree } else { else_tree },
                state,
                node_name,
            )
        }
    }
}
