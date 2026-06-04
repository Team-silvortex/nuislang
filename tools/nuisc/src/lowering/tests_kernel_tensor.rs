use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_kernel_tensor_inspect_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "shape"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "rows"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "cols"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "row"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "col"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "element_at"));
}

#[test]
fn lowers_kernel_tensor_map_zip_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "add_scalar"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "mul_scalar"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "relu"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "mul"));
}

#[test]
fn lowers_kernel_tensor_reshape_primitive_into_kernel_node() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reshape"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "element_at"));
}

#[test]
fn lowers_kernel_tensor_broadcast_primitive_into_kernel_node() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "broadcast"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "element_at"));
}

#[test]
fn lowers_kernel_tensor_reduction_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reduce_max"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reduce_mean"));
}

#[test]
fn lowers_kernel_tensor_reduce_axis_primitive_into_kernel_node() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reduce_sum_axis"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "element_at"));
}

#[test]
fn lowers_kernel_tensor_reduce_axis_family_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reduce_max_axis"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reduce_mean_axis"));
}

#[test]
fn lowers_kernel_tensor_select_axis_family_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "argmax_axis"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "argmin_axis"));
}

#[test]
fn lowers_kernel_tensor_topk_axis_primitive_into_kernel_node() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "topk_axis"));
}

#[test]
fn lowers_kernel_tensor_sort_axis_primitive_into_kernel_node() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "sort_axis"));
}

#[test]
fn lowers_kernel_tensor_map_axis_primitive_into_kernel_node() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "relu_axis"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "add_scalar_axis"));
}

#[test]
fn lowers_kernel_tensor_selection_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "argmax"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "argmin"));
}

#[test]
fn lowers_kernel_tensor_order_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "sort"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "topk"));
}
