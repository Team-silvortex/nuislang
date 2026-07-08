use super::*;

#[test]
fn lowers_kernel_tensor_primitives_into_kernel_nodes() {
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
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "tensor"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "matmul"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "add_bias"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "relu"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reduce_sum"));
}
