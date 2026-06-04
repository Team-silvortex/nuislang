use super::*;

pub(super) fn lower_kernel_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::KernelProfileBindCoreRef { unit } => Some(lower_project_profile_ref(
            state,
            "kernel",
            unit,
            "bind_core",
        )),
        NirExpr::KernelProfileQueueDepthRef { unit } => Some(lower_project_profile_ref(
            state,
            "kernel",
            unit,
            "queue_depth",
        )),
        NirExpr::KernelProfileBatchLanesRef { unit } => Some(lower_project_profile_ref(
            state,
            "kernel",
            unit,
            "batch_lanes",
        )),
        NirExpr::KernelResult { value, state: flow } => Some(lower_result_observe_node(
            state,
            bindings,
            ResultLoweringDomain::Kernel,
            value,
            "kernel_result",
            flow.render(),
        )),
        NirExpr::KernelConfigReady(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Kernel,
            result,
            "kernel_config_ready",
            "is_config_ready",
        )),
        NirExpr::KernelValue(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Kernel,
            result,
            "kernel_value",
            "value",
        )),
        NirExpr::KernelTensor {
            rows,
            cols,
            elements_csv,
        } => Some(Ok(lower_kernel_tensor(*rows, *cols, elements_csv, state))),
        NirExpr::KernelShape(input) => Some(lower_kernel_unary(
            input,
            "kernel_shape",
            "shape",
            state,
            bindings,
        )),
        NirExpr::KernelRows(input) => Some(lower_kernel_unary(
            input,
            "kernel_rows",
            "rows",
            state,
            bindings,
        )),
        NirExpr::KernelCols(input) => Some(lower_kernel_unary(
            input,
            "kernel_cols",
            "cols",
            state,
            bindings,
        )),
        NirExpr::KernelRow(input) => Some(lower_kernel_unary(
            input,
            "kernel_row",
            "row",
            state,
            bindings,
        )),
        NirExpr::KernelCol(input) => Some(lower_kernel_unary(
            input,
            "kernel_col",
            "col",
            state,
            bindings,
        )),
        NirExpr::KernelElementAt { input, row, col } => {
            Some(lower_kernel_element_at(input, row, col, state, bindings))
        }
        NirExpr::KernelReshape { input, rows, cols } => {
            Some(lower_kernel_reshape(input, *rows, *cols, state, bindings))
        }
        NirExpr::KernelBroadcast { input, rows, cols } => {
            Some(lower_kernel_broadcast(input, *rows, *cols, state, bindings))
        }
        NirExpr::KernelMap { input, op, scalar } => Some(lower_kernel_map(
            input,
            op,
            scalar.as_deref(),
            state,
            bindings,
        )),
        NirExpr::KernelMapAxis {
            input,
            axis,
            op,
            scalar,
        } => Some(lower_kernel_map_axis(
            input,
            axis.render(),
            op,
            scalar.as_deref(),
            state,
            bindings,
        )),
        NirExpr::KernelZip { lhs, rhs, op } => {
            Some(lower_kernel_zip(lhs, rhs, op, state, bindings))
        }
        NirExpr::KernelMatmul { lhs, rhs } => Some(lower_kernel_binary(
            lhs,
            rhs,
            "kernel_matmul",
            "matmul",
            state,
            bindings,
        )),
        NirExpr::KernelAddBias { input, bias } => Some(lower_kernel_binary(
            input,
            bias,
            "kernel_add_bias",
            "add_bias",
            state,
            bindings,
        )),
        NirExpr::KernelRelu(input) => Some(lower_kernel_unary(
            input,
            "kernel_relu",
            "relu",
            state,
            bindings,
        )),
        NirExpr::KernelReduceSum(input) => Some(lower_kernel_unary(
            input,
            "kernel_reduce_sum",
            "reduce_sum",
            state,
            bindings,
        )),
        NirExpr::KernelReduceSumAxis { input, axis } => Some(lower_kernel_axis_unary(
            input,
            axis.render(),
            "kernel_reduce_sum_axis",
            "reduce_sum_axis",
            state,
            bindings,
        )),
        NirExpr::KernelReduceMax(input) => Some(lower_kernel_unary(
            input,
            "kernel_reduce_max",
            "reduce_max",
            state,
            bindings,
        )),
        NirExpr::KernelReduceMaxAxis { input, axis } => Some(lower_kernel_axis_unary(
            input,
            axis.render(),
            "kernel_reduce_max_axis",
            "reduce_max_axis",
            state,
            bindings,
        )),
        NirExpr::KernelReduceMean(input) => Some(lower_kernel_unary(
            input,
            "kernel_reduce_mean",
            "reduce_mean",
            state,
            bindings,
        )),
        NirExpr::KernelReduceMeanAxis { input, axis } => Some(lower_kernel_axis_unary(
            input,
            axis.render(),
            "kernel_reduce_mean_axis",
            "reduce_mean_axis",
            state,
            bindings,
        )),
        NirExpr::KernelArgmax(input) => Some(lower_kernel_unary(
            input,
            "kernel_argmax",
            "argmax",
            state,
            bindings,
        )),
        NirExpr::KernelArgmaxAxis { input, axis } => Some(lower_kernel_axis_unary(
            input,
            axis.render(),
            "kernel_argmax_axis",
            "argmax_axis",
            state,
            bindings,
        )),
        NirExpr::KernelArgmin(input) => Some(lower_kernel_unary(
            input,
            "kernel_argmin",
            "argmin",
            state,
            bindings,
        )),
        NirExpr::KernelArgminAxis { input, axis } => Some(lower_kernel_axis_unary(
            input,
            axis.render(),
            "kernel_argmin_axis",
            "argmin_axis",
            state,
            bindings,
        )),
        NirExpr::KernelSort(input) => Some(lower_kernel_unary(
            input,
            "kernel_sort",
            "sort",
            state,
            bindings,
        )),
        NirExpr::KernelSortAxis { input, axis } => Some(lower_kernel_axis_unary(
            input,
            axis.render(),
            "kernel_sort_axis",
            "sort_axis",
            state,
            bindings,
        )),
        NirExpr::KernelTopk { input, k } => {
            Some(lower_kernel_topk(input, *k, None, state, bindings))
        }
        NirExpr::KernelTopkAxis { input, axis, k } => Some(lower_kernel_topk(
            input,
            *k,
            Some(axis.render()),
            state,
            bindings,
        )),
        _ => None,
    }
}

fn ensure_kernel(state: &mut LoweringState<'_>) {
    ensure_kernel_resource(state.yir);
}

fn lower_kernel_tensor(
    rows: i64,
    cols: i64,
    elements_csv: &str,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_kernel(state);
    let name = next_name(state, "kernel_tensor");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: "tensor".to_owned(),
            args: vec![rows.to_string(), cols.to_string(), elements_csv.to_owned()],
        },
    });
    name
}

fn lower_kernel_unary(
    input: &NirExpr,
    prefix: &str,
    instruction: &str,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![input_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}

fn lower_kernel_axis_unary(
    input: &NirExpr,
    axis: &str,
    prefix: &str,
    instruction: &str,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![input_name.clone(), axis.to_owned()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}

fn lower_kernel_binary(
    lhs: &NirExpr,
    rhs: &NirExpr,
    prefix: &str,
    instruction: &str,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let lhs_name = lower_expr(lhs, state, bindings)?;
    let rhs_name = lower_expr(rhs, state, bindings)?;
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![lhs_name.clone(), rhs_name.clone()],
        },
    });
    push_dep_edges(state, &lhs_name, &name);
    push_dep_edges(state, &rhs_name, &name);
    Ok(name)
}

fn lower_kernel_element_at(
    input: &NirExpr,
    row: &NirExpr,
    col: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let row_name = lower_expr(row, state, bindings)?;
    let col_name = lower_expr(col, state, bindings)?;
    let name = next_name(state, "kernel_element_at");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: "element_at".to_owned(),
            args: vec![input_name.clone(), row_name.clone(), col_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    push_dep_edges(state, &row_name, &name);
    push_dep_edges(state, &col_name, &name);
    Ok(name)
}

fn lower_kernel_reshape(
    input: &NirExpr,
    rows: i64,
    cols: i64,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, "kernel_reshape");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: "reshape".to_owned(),
            args: vec![input_name.clone(), rows.to_string(), cols.to_string()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}

fn lower_kernel_broadcast(
    input: &NirExpr,
    rows: i64,
    cols: i64,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, "kernel_broadcast");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: "broadcast".to_owned(),
            args: vec![input_name.clone(), rows.to_string(), cols.to_string()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}

fn lower_kernel_map(
    input: &NirExpr,
    op: &NirKernelMapOp,
    scalar: Option<&NirExpr>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let mut args = vec![input_name.clone()];
    let mut scalar_name = None;
    if let Some(scalar) = scalar {
        let lowered = lower_expr(scalar, state, bindings)?;
        args.push(lowered.clone());
        scalar_name = Some(lowered);
    }
    let name = next_name(state, "kernel_map");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: op.instruction().to_owned(),
            args,
        },
    });
    push_dep_edges(state, &input_name, &name);
    if let Some(scalar_name) = scalar_name {
        push_dep_edges(state, &scalar_name, &name);
    }
    Ok(name)
}

fn lower_kernel_map_axis(
    input: &NirExpr,
    axis: &str,
    op: &NirKernelMapOp,
    scalar: Option<&NirExpr>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let mut args = vec![input_name.clone(), axis.to_owned()];
    let mut scalar_name = None;
    if let Some(scalar) = scalar {
        let lowered = lower_expr(scalar, state, bindings)?;
        args.push(lowered.clone());
        scalar_name = Some(lowered);
    }
    let name = next_name(state, "kernel_map_axis");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: match op {
                NirKernelMapOp::Relu => "relu_axis".to_owned(),
                NirKernelMapOp::AddScalar => "add_scalar_axis".to_owned(),
                NirKernelMapOp::MulScalar => "mul_scalar_axis".to_owned(),
            },
            args,
        },
    });
    push_dep_edges(state, &input_name, &name);
    if let Some(scalar_name) = scalar_name {
        push_dep_edges(state, &scalar_name, &name);
    }
    Ok(name)
}

fn lower_kernel_zip(
    lhs: &NirExpr,
    rhs: &NirExpr,
    op: &nuis_semantics::model::NirKernelZipOp,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let lhs_name = lower_expr(lhs, state, bindings)?;
    let rhs_name = lower_expr(rhs, state, bindings)?;
    let name = next_name(state, "kernel_zip");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: op.instruction().to_owned(),
            args: vec![lhs_name.clone(), rhs_name.clone()],
        },
    });
    push_dep_edges(state, &lhs_name, &name);
    push_dep_edges(state, &rhs_name, &name);
    Ok(name)
}

fn lower_kernel_topk(
    input: &NirExpr,
    k: i64,
    axis: Option<&str>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_kernel(state);
    let input_name = lower_expr(input, state, bindings)?;
    let (prefix, instruction, mut args) = match axis {
        Some(axis) => (
            "kernel_topk_axis",
            "topk_axis",
            vec![input_name.clone(), k.to_string(), axis.to_owned()],
        ),
        None => (
            "kernel_topk",
            "topk",
            vec![input_name.clone(), k.to_string()],
        ),
    };
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "kernel0".to_owned(),
        op: Operation {
            module: "kernel".to_owned(),
            instruction: instruction.to_owned(),
            args: std::mem::take(&mut args),
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}
