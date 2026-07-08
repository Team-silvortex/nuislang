use super::*;

#[test]
fn lowers_explicit_kernel_result_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lanes: KernelResult<i64> = kernel_result(kernel_profile_batch_lanes("KernelUnit"));
            let ready: bool = kernel_config_ready(lanes);
            let value: i64 = kernel_value(lanes);
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::KernelResult { state, .. },
            ..
        }) if ty.render() == "KernelResult<i64>"
            && matches!(state, NirKernelFlowState::ConfigReady)
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::KernelConfigReady(_),
            ..
        }) if ty.render() == "bool"
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::KernelValue(_),
            ..
        }) if ty.render() == "i64"
    ));
}

#[test]
fn lowers_explicit_kernel_result_helpers_from_tensor_reductions() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let total: KernelResult<i64> = kernel_result(kernel_reduce_sum(input));
            let peak: KernelResult<i64> = kernel_result(kernel_reduce_max(input));
            let avg: KernelResult<i64> = kernel_result(kernel_reduce_mean(input));
            return kernel_value(total) + kernel_value(peak) + kernel_value(avg);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::KernelResult { value, state },
            ..
        } => {
            ty.render() == "KernelResult<i64>"
                && matches!(state, NirKernelFlowState::ConfigReady)
                && matches!(value.as_ref(), NirExpr::KernelReduceSum(_))
        }
        _ => false,
    }));
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            value: NirExpr::KernelResult { value, .. },
            ..
        } => matches!(value.as_ref(), NirExpr::KernelReduceMax(_)),
        _ => false,
    }));
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            value: NirExpr::KernelResult { value, .. },
            ..
        } => matches!(value.as_ref(), NirExpr::KernelReduceMean(_)),
        _ => false,
    }));
}

#[test]
fn lowers_explicit_kernel_tensor_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(1, 3, "2,4,6");
            let weights = kernel_tensor(3, 2, "1,-2,3,0,2,1");
            let bias = kernel_tensor(1, 2, "-4,3");
            let projected = kernel_matmul(input, weights);
            let shifted = kernel_add_bias(projected, bias);
            let activated = kernel_relu(shifted);
            return kernel_reduce_sum(activated);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelTensor { .. },
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelMatmul { .. },
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelAddBias { .. },
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelRelu(_),
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelReduceSum(_))))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_inspect_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(1, 3, "2,4,6");
            let layout = kernel_shape(input);
            let rows: i64 = kernel_rows(input);
            let cols: i64 = kernel_cols(input);
            let first_row = kernel_row(input);
            let first_col = kernel_col(input);
            return kernel_element_at(first_row, 0, 1) + rows + cols + kernel_element_at(first_col, 0, 0);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelShape(_),
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelRows(_),
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelCols(_),
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelRow(_),
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelCol(_),
            ..
        }
    )));
    assert!(function
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
}
