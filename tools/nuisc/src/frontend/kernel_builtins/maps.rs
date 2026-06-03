use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirKernelAxis, NirKernelMapOp, NirKernelZipOp, NirStructDef, NirTypeRef,
};

use super::super::{i64_type, lower_expr, FunctionSignature};

pub(super) fn lower_kernel_map_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "kernel_map" => match args {
            [input, op] => {
                let op = parse_kernel_map_op(op, "kernel_map", false)?;
                NirExpr::KernelMap {
                    input: Box::new(lower_expr(
                        input,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        None,
                    )?),
                    op,
                    scalar: None,
                }
            }
            [input, op, scalar] => {
                let op = parse_kernel_map_op(op, "kernel_map", true)?;
                NirExpr::KernelMap {
                    input: Box::new(lower_expr(
                        input,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        None,
                    )?),
                    op,
                    scalar: Some(Box::new(lower_expr(
                        scalar,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )?)),
                }
            }
            _ => return Err("kernel_map(...) expects 2 or 3 args".to_owned()),
        },
        "kernel_map_axis" => match args {
            [input, axis, op] => {
                let axis = parse_kernel_axis(axis, "kernel_map_axis")?;
                let op = parse_kernel_map_op(op, "kernel_map_axis", false)?;
                NirExpr::KernelMapAxis {
                    input: Box::new(lower_expr(
                        input,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        None,
                    )?),
                    axis,
                    op,
                    scalar: None,
                }
            }
            [input, axis, op, scalar] => {
                let axis = parse_kernel_axis(axis, "kernel_map_axis")?;
                let op = parse_kernel_map_op(op, "kernel_map_axis", true)?;
                NirExpr::KernelMapAxis {
                    input: Box::new(lower_expr(
                        input,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        None,
                    )?),
                    axis,
                    op,
                    scalar: Some(Box::new(lower_expr(
                        scalar,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )?)),
                }
            }
            _ => return Err("kernel_map_axis(...) expects 3 or 4 args".to_owned()),
        },
        "kernel_zip" => {
            let [lhs, rhs, op] = args else {
                return Err("kernel_zip(...) expects 3 args".to_owned());
            };
            let op = parse_kernel_zip_op(op)?;
            NirExpr::KernelZip {
                lhs: Box::new(lower_expr(
                    lhs,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                rhs: Box::new(lower_expr(
                    rhs,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                op,
            }
        }
        "kernel_matmul" => {
            let [lhs, rhs] = args else {
                return Err("kernel_matmul(...) expects 2 args".to_owned());
            };
            NirExpr::KernelMatmul {
                lhs: Box::new(lower_expr(
                    lhs,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                rhs: Box::new(lower_expr(
                    rhs,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
            }
        }
        "kernel_add_bias" => {
            let [input, bias] = args else {
                return Err("kernel_add_bias(...) expects 2 args".to_owned());
            };
            NirExpr::KernelAddBias {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                bias: Box::new(lower_expr(
                    bias,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
            }
        }
        "kernel_relu" => {
            let [input] = args else {
                return Err("kernel_relu(...) expects 1 arg".to_owned());
            };
            NirExpr::KernelRelu(Box::new(lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?))
        }
        "kernel_reduce_sum" => {
            let [input] = args else {
                return Err("kernel_reduce_sum(...) expects 1 arg".to_owned());
            };
            NirExpr::KernelReduceSum(Box::new(lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?))
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

fn parse_kernel_map_op(
    op: &AstExpr,
    callee: &str,
    with_scalar: bool,
) -> Result<NirKernelMapOp, String> {
    let AstExpr::Text(op_name) = op else {
        return Err(format!("{callee}(...) op must be a string literal"));
    };
    match (op_name.as_str(), with_scalar) {
        ("relu", false) => Ok(NirKernelMapOp::Relu),
        ("add_scalar", true) => Ok(NirKernelMapOp::AddScalar),
        ("mul_scalar", true) => Ok(NirKernelMapOp::MulScalar),
        ("add_scalar" | "mul_scalar", false) => Err(format!(
            "{callee}(..., \"{}\") expects a {}scalar arg",
            op_name,
            if callee == "kernel_map_axis" {
                "fourth "
            } else {
                "third "
            }
        )),
        ("relu", true) => Err(format!(
            "{callee}(..., \"relu\") does not accept a scalar arg"
        )),
        _ => Err(format!(
            "{callee}(...) unsupported op `{}`; expected relu/add_scalar/mul_scalar",
            op_name
        )),
    }
}

fn parse_kernel_zip_op(op: &AstExpr) -> Result<NirKernelZipOp, String> {
    let AstExpr::Text(op_name) = op else {
        return Err("kernel_zip(...) op must be a string literal".to_owned());
    };
    match op_name.as_str() {
        "add" => Ok(NirKernelZipOp::Add),
        "mul" => Ok(NirKernelZipOp::Mul),
        _ => Err(format!(
            "kernel_zip(...) unsupported op `{}`; expected add/mul",
            op_name
        )),
    }
}
