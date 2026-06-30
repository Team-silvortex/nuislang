use super::*;

pub(super) fn render_kernel_nir_expr(value: &NirExpr) -> Option<String> {
    let rendered = match value {
        NirExpr::KernelProfileBindCoreRef { unit } => {
            format!("kernel_profile_bind_core(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelProfileQueueDepthRef { unit } => {
            format!("kernel_profile_queue_depth(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelProfileBatchLanesRef { unit } => {
            format!("kernel_profile_batch_lanes(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelResult { value, .. } => {
            format!("kernel_result({})", render_nir_expr(value))
        }
        NirExpr::KernelConfigReady(result) => {
            format!("kernel_config_ready({})", render_nir_expr(result))
        }
        NirExpr::KernelValue(result) => format!("kernel_value({})", render_nir_expr(result)),
        NirExpr::KernelTensor {
            rows,
            cols,
            elements_csv,
        } => format!(
            "kernel_tensor({}, {}, \"{}\")",
            rows,
            cols,
            escape_debug(elements_csv)
        ),
        NirExpr::KernelShape(input) => format!("kernel_shape({})", render_nir_expr(input)),
        NirExpr::KernelRows(input) => format!("kernel_rows({})", render_nir_expr(input)),
        NirExpr::KernelCols(input) => format!("kernel_cols({})", render_nir_expr(input)),
        NirExpr::KernelRow(input) => format!("kernel_row({})", render_nir_expr(input)),
        NirExpr::KernelCol(input) => format!("kernel_col({})", render_nir_expr(input)),
        NirExpr::KernelElementAt { input, row, col } => format!(
            "kernel_element_at({}, {}, {})",
            render_nir_expr(input),
            render_nir_expr(row),
            render_nir_expr(col)
        ),
        NirExpr::KernelReshape { input, rows, cols } => format!(
            "kernel_reshape({}, {}, {})",
            render_nir_expr(input),
            rows,
            cols
        ),
        NirExpr::KernelBroadcast { input, rows, cols } => format!(
            "kernel_broadcast({}, {}, {})",
            render_nir_expr(input),
            rows,
            cols
        ),
        NirExpr::KernelMap { input, op, scalar } => match scalar {
            Some(scalar) => format!(
                "kernel_map({}, \"{}\", {})",
                render_nir_expr(input),
                op.render(),
                render_nir_expr(scalar)
            ),
            None => format!(
                "kernel_map({}, \"{}\")",
                render_nir_expr(input),
                op.render()
            ),
        },
        NirExpr::KernelMapAxis {
            input,
            axis,
            op,
            scalar,
        } => match scalar {
            Some(scalar) => format!(
                "kernel_map_axis({}, \"{}\", \"{}\", {})",
                render_nir_expr(input),
                axis.render(),
                op.render(),
                render_nir_expr(scalar)
            ),
            None => format!(
                "kernel_map_axis({}, \"{}\", \"{}\")",
                render_nir_expr(input),
                axis.render(),
                op.render()
            ),
        },
        NirExpr::KernelZip { lhs, rhs, op } => format!(
            "kernel_zip({}, {}, \"{}\")",
            render_nir_expr(lhs),
            render_nir_expr(rhs),
            op.render()
        ),
        NirExpr::KernelMatmul { lhs, rhs } => format!(
            "kernel_matmul({}, {})",
            render_nir_expr(lhs),
            render_nir_expr(rhs)
        ),
        NirExpr::KernelAddBias { input, bias } => format!(
            "kernel_add_bias({}, {})",
            render_nir_expr(input),
            render_nir_expr(bias)
        ),
        NirExpr::KernelRelu(input) => format!("kernel_relu({})", render_nir_expr(input)),
        NirExpr::KernelReduceSum(input) => {
            format!("kernel_reduce_sum({})", render_nir_expr(input))
        }
        NirExpr::KernelReduceSumAxis { input, axis } => format!(
            "kernel_reduce_sum_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelReduceMaxAxis { input, axis } => format!(
            "kernel_reduce_max_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelReduceMeanAxis { input, axis } => format!(
            "kernel_reduce_mean_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelReduceMax(input) => {
            format!("kernel_reduce_max({})", render_nir_expr(input))
        }
        NirExpr::KernelReduceMean(input) => {
            format!("kernel_reduce_mean({})", render_nir_expr(input))
        }
        NirExpr::KernelArgmaxAxis { input, axis } => format!(
            "kernel_argmax_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelArgminAxis { input, axis } => format!(
            "kernel_argmin_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelArgmax(input) => format!("kernel_argmax({})", render_nir_expr(input)),
        NirExpr::KernelArgmin(input) => format!("kernel_argmin({})", render_nir_expr(input)),
        NirExpr::KernelSort(input) => format!("kernel_sort({})", render_nir_expr(input)),
        NirExpr::KernelSortAxis { input, axis } => format!(
            "kernel_sort_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelTopk { input, k } => {
            format!("kernel_topk({}, {})", render_nir_expr(input), k)
        }
        NirExpr::KernelTopkAxis { input, axis, k } => format!(
            "kernel_topk_axis({}, \"{}\", {})",
            render_nir_expr(input),
            axis.render(),
            k
        ),
        _ => return None,
    };
    Some(rendered)
}
