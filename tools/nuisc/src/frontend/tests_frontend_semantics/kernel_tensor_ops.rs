use super::*;

#[test]
fn lowers_explicit_kernel_tensor_map_zip_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(1, 3, "2,4,6");
            let lifted = kernel_map(input, "add_scalar", 3);
            let scaled = kernel_map(lifted, "mul_scalar", 2);
            let activated = kernel_map(scaled, "relu");
            let mask = kernel_tensor(1, 3, "1,0,1");
            let mixed = kernel_zip(activated, mask, "mul");
            return kernel_reduce_sum(mixed);
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
            value: NirExpr::KernelMap { .. },
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelZip { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelReduceSum(_))))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_reshape_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let reshaped = kernel_reshape(input, 3, 2);
            return kernel_element_at(reshaped, 2, 1);
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
            value: NirExpr::KernelReshape { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_broadcast_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(1, 3, "2,4,6");
            let widened = kernel_broadcast(input, 2, 3);
            return kernel_element_at(widened, 1, 2);
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
            value: NirExpr::KernelBroadcast { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_reduction_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let maxed: i64 = kernel_reduce_max(input);
            return maxed + kernel_reduce_mean(input);
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
            value: NirExpr::KernelReduceMax(_),
            ..
        }
    )));
    assert!(function
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
}

#[test]
fn lowers_explicit_kernel_tensor_selection_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let hi: i64 = kernel_argmax(input);
            return hi + kernel_argmin(input);
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
            value: NirExpr::KernelArgmax(_),
            ..
        }
    )));
    assert!(function
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
}

#[test]
fn lowers_explicit_kernel_tensor_reduce_axis_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let row_sums = kernel_reduce_sum_axis(input, "rows");
            return kernel_element_at(row_sums, 0, 1);
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
            value: NirExpr::KernelReduceSumAxis { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_reduce_axis_family_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let row_max = kernel_reduce_max_axis(input, "rows");
            let col_mean = kernel_reduce_mean_axis(input, "cols");
            return kernel_element_at(row_max, 0, 0) + kernel_element_at(col_mean, 0, 1);
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
            value: NirExpr::KernelReduceMaxAxis { .. },
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelReduceMeanAxis { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_select_axis_family_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let row_hi = kernel_argmax_axis(input, "rows");
            let col_lo = kernel_argmin_axis(input, "cols");
            return kernel_element_at(row_hi, 0, 1) + kernel_element_at(col_lo, 0, 2);
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
            value: NirExpr::KernelArgmaxAxis { .. },
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelArgminAxis { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_topk_axis_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let top2_rows = kernel_topk_axis(input, "rows", 2);
            return kernel_element_at(top2_rows, 0, 1);
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
            value: NirExpr::KernelTopkAxis { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_map_axis_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "-2,4,-6,1,-3,5");
            let activated = kernel_map_axis(input, "rows", "relu");
            let lifted = kernel_map_axis(activated, "cols", "add_scalar", 2);
            return kernel_element_at(lifted, 0, 0);
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
            value: NirExpr::KernelMapAxis { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_sort_axis_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let sorted_rows = kernel_sort_axis(input, "rows");
            return kernel_element_at(sorted_rows, 0, 1);
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
            value: NirExpr::KernelSortAxis { .. },
            ..
        }
    )));
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
    ));
}

#[test]
fn lowers_explicit_kernel_tensor_order_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
            let sorted = kernel_sort(input);
            let top2 = kernel_topk(input, 2);
            return kernel_element_at(sorted, 0, 0) + kernel_element_at(top2, 0, 1);
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
            value: NirExpr::KernelSort(_),
            ..
        }
    )));
    assert!(function.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            value: NirExpr::KernelTopk { .. },
            ..
        }
    )));
    assert!(function
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
}
