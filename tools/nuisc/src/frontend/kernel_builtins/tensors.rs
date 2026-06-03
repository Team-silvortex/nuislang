use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirKernelAxis, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, FunctionSignature};

pub(super) fn lower_kernel_tensor_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "kernel_tensor" => {
            let [rows, cols, elements] = args else {
                return Err("kernel_tensor(...) expects 3 args".to_owned());
            };
            let AstExpr::Int(rows) = rows else {
                return Err("kernel_tensor(...) rows must be an integer literal".to_owned());
            };
            let AstExpr::Int(cols) = cols else {
                return Err("kernel_tensor(...) cols must be an integer literal".to_owned());
            };
            let AstExpr::Text(elements) = elements else {
                return Err("kernel_tensor(...) elements must be a CSV string literal".to_owned());
            };
            NirExpr::KernelTensor {
                rows: *rows,
                cols: *cols,
                elements_csv: elements.clone(),
            }
        }
        "kernel_shape" => lower_unary_kernel_expr(
            "kernel_shape",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelShape,
        )?,
        "kernel_rows" => lower_unary_kernel_expr(
            "kernel_rows",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelRows,
        )?,
        "kernel_cols" => lower_unary_kernel_expr(
            "kernel_cols",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelCols,
        )?,
        "kernel_row" => lower_unary_kernel_expr(
            "kernel_row",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelRow,
        )?,
        "kernel_col" => lower_unary_kernel_expr(
            "kernel_col",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelCol,
        )?,
        "kernel_element_at" => {
            let [input, row, col] = args else {
                return Err("kernel_element_at(...) expects 3 args".to_owned());
            };
            NirExpr::KernelElementAt {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                row: Box::new(lower_expr(
                    row,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                col: Box::new(lower_expr(
                    col,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            }
        }
        "kernel_reshape" => {
            let [input, rows, cols] = args else {
                return Err("kernel_reshape(...) expects 3 args".to_owned());
            };
            let AstExpr::Int(rows) = rows else {
                return Err("kernel_reshape(...) rows must be an integer literal".to_owned());
            };
            let AstExpr::Int(cols) = cols else {
                return Err("kernel_reshape(...) cols must be an integer literal".to_owned());
            };
            NirExpr::KernelReshape {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                rows: *rows,
                cols: *cols,
            }
        }
        "kernel_broadcast" => {
            let [input, rows, cols] = args else {
                return Err("kernel_broadcast(...) expects 3 args".to_owned());
            };
            let AstExpr::Int(rows) = rows else {
                return Err("kernel_broadcast(...) rows must be an integer literal".to_owned());
            };
            let AstExpr::Int(cols) = cols else {
                return Err("kernel_broadcast(...) cols must be an integer literal".to_owned());
            };
            NirExpr::KernelBroadcast {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                rows: *rows,
                cols: *cols,
            }
        }
        "kernel_reduce_max" => lower_unary_kernel_expr(
            "kernel_reduce_max",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelReduceMax,
        )?,
        "kernel_reduce_mean" => lower_unary_kernel_expr(
            "kernel_reduce_mean",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelReduceMean,
        )?,
        "kernel_reduce_max_axis" => lower_axis_kernel_expr(
            "kernel_reduce_max_axis",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            |input, axis| NirExpr::KernelReduceMaxAxis { input, axis },
        )?,
        "kernel_reduce_mean_axis" => lower_axis_kernel_expr(
            "kernel_reduce_mean_axis",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            |input, axis| NirExpr::KernelReduceMeanAxis { input, axis },
        )?,
        "kernel_reduce_sum_axis" => lower_axis_kernel_expr(
            "kernel_reduce_sum_axis",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            |input, axis| NirExpr::KernelReduceSumAxis { input, axis },
        )?,
        "kernel_argmax" => lower_unary_kernel_expr(
            "kernel_argmax",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelArgmax,
        )?,
        "kernel_argmax_axis" => lower_axis_kernel_expr(
            "kernel_argmax_axis",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            |input, axis| NirExpr::KernelArgmaxAxis { input, axis },
        )?,
        "kernel_argmin" => lower_unary_kernel_expr(
            "kernel_argmin",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelArgmin,
        )?,
        "kernel_argmin_axis" => lower_axis_kernel_expr(
            "kernel_argmin_axis",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            |input, axis| NirExpr::KernelArgminAxis { input, axis },
        )?,
        "kernel_sort" => lower_unary_kernel_expr(
            "kernel_sort",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            NirExpr::KernelSort,
        )?,
        "kernel_sort_axis" => lower_axis_kernel_expr(
            "kernel_sort_axis",
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
            |input, axis| NirExpr::KernelSortAxis { input, axis },
        )?,
        "kernel_topk" => {
            let [input, k] = args else {
                return Err("kernel_topk(...) expects 2 args".to_owned());
            };
            let AstExpr::Int(k) = k else {
                return Err("kernel_topk(...) k must be an integer literal".to_owned());
            };
            NirExpr::KernelTopk {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                k: *k,
            }
        }
        "kernel_topk_axis" => {
            let [input, axis, k] = args else {
                return Err("kernel_topk_axis(...) expects 3 args".to_owned());
            };
            let axis = parse_kernel_axis(axis, "kernel_topk_axis")?;
            let AstExpr::Int(k) = k else {
                return Err("kernel_topk_axis(...) k must be an integer literal".to_owned());
            };
            NirExpr::KernelTopkAxis {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                axis,
                k: *k,
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

fn parse_kernel_axis(axis: &AstExpr, callee: &str) -> Result<NirKernelAxis, String> {
    let AstExpr::Text(axis_name) = axis else {
        return Err(format!("{callee}(...) axis must be a string literal"));
    };
    match axis_name.as_str() {
        "rows" => Ok(NirKernelAxis::Rows),
        "cols" => Ok(NirKernelAxis::Cols),
        _ => Err(format!(
            "{callee}(...) unsupported axis `{}`; expected rows/cols",
            axis_name
        )),
    }
}

fn lower_unary_kernel_expr(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap: fn(Box<NirExpr>) -> NirExpr,
) -> Result<NirExpr, String> {
    let [input] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    Ok(wrap(Box::new(lower_expr(
        input,
        current_domain,
        bindings,
        signatures,
        struct_table,
        None,
    )?)))
}

fn lower_axis_kernel_expr(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap: fn(Box<NirExpr>, NirKernelAxis) -> NirExpr,
) -> Result<NirExpr, String> {
    let [input, axis] = args else {
        return Err(format!("{callee}(...) expects 2 args"));
    };
    Ok(wrap(
        Box::new(lower_expr(
            input,
            current_domain,
            bindings,
            signatures,
            struct_table,
            None,
        )?),
        parse_kernel_axis(axis, callee)?,
    ))
}
