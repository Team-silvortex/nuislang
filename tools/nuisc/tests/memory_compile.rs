use std::path::Path;

use nuis_semantics::model::{NirExpr, NirStmt};

fn compiled_source(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_source_path(Path::new(path))
        .unwrap_or_else(|error| panic!("source `{path}` should compile: {error}"))
}

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
}

#[test]
fn compiles_hello_glm_memory_source() {
    let source = Path::new("/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_glm.ns");
    nuisc::pipeline::compile_source_path(source).expect("hello_glm memory source should compile");
}

#[test]
fn lowers_hello_glm_memory_source_with_structural_pointer_shape() {
    let artifacts =
        compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_glm.ns");

    let alloc_nodes = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "alloc_node")
        .count();
    let borrows = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow")
        .count();
    let load_nexts = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "load_next")
        .count();
    let load_values = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "load_value")
        .count();
    let borrow_ends = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .count();

    assert!(
        alloc_nodes >= 2,
        "expected structural pointer allocation path"
    );
    assert!(borrows >= 2, "expected structural borrow path");
    assert!(load_nexts >= 1, "expected structural next-link load path");
    assert!(load_values >= 2, "expected structural payload load path");
    assert!(borrow_ends >= 2, "expected explicit borrow closure path");
}

#[test]
fn compiles_hello_borrow_end_memory_source() {
    let source =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns");
    nuisc::pipeline::compile_source_path(source)
        .expect("hello_borrow_end memory source should compile");
}

#[test]
fn lowers_hello_borrow_end_memory_source_with_borrow_end_then_owner_write_shape() {
    let artifacts =
        compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns");

    let lowered_ops = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu")
        .map(|node| node.op.instruction.as_str())
        .collect::<Vec<_>>();
    let borrow_end_index = lowered_ops
        .iter()
        .position(|op| *op == "borrow_end")
        .expect("expected borrow_end op");
    let store_value_index = lowered_ops
        .iter()
        .position(|op| *op == "store_value")
        .expect("expected store_value op");
    assert!(
        borrow_end_index < store_value_index,
        "expected borrow_end to lower before owner write, got {lowered_ops:?}"
    );
}

#[test]
fn compiles_hello_ref_struct_source() {
    let source =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns");
    nuisc::pipeline::compile_source_path(source).expect("hello_ref_struct source should compile");
}

#[test]
fn lowers_hello_ref_struct_source_with_ref_field_shape() {
    let artifacts =
        compiled_source("/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns");

    let pair = artifacts
        .nir
        .structs
        .iter()
        .find(|definition| definition.name == "Pair")
        .expect("expected Pair struct");
    assert_eq!(pair.fields.len(), 2);
    assert_eq!(pair.fields[0].ty.render(), "ref Node");
    assert_eq!(pair.fields[1].ty.render(), "ref Node");

    let borrow_count = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow")
        .count();
    assert!(borrow_count >= 2, "expected ref-struct field borrow path");
}

#[test]
fn compiles_hello_buffer_addressing_memory_source() {
    let source = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_buffer_addressing.ns",
    );
    nuisc::pipeline::compile_source_path(source)
        .expect("hello_buffer_addressing memory source should compile");
}

#[test]
fn lowers_hello_buffer_addressing_memory_source_with_buffer_address_shape() {
    let artifacts = compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_buffer_addressing.ns",
    );

    let alloc_buffers = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "alloc_buffer")
        .count();
    let store_ats = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "store_at")
        .count();
    let load_ats = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "load_at")
        .count();

    assert!(alloc_buffers >= 1, "expected buffer allocation path");
    assert!(store_ats >= 4, "expected staged buffer store path");
    assert!(load_ats >= 4, "expected staged buffer load path");
}

#[test]
fn lowers_hello_buffer_addressing_memory_source_with_ref_buffer_nir_shape() {
    let artifacts = compiled_source(
        "/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_buffer_addressing.ns",
    );

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::AllocBuffer { .. },
            } if name == "scratch" && ty.render() == "ref Buffer"
        )
    }));
}

#[test]
fn compiles_stdin_runtime_demo_project_with_host_buffer_handle_bridge() {
    compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/stdin_runtime_demo",
    );
}

#[test]
fn compiles_file_runtime_demo_project_with_host_buffer_handle_bridge() {
    compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/filesystem/file_runtime_demo",
    );
}

#[test]
fn compiles_stdin_file_input_runtime_demo_project_with_host_buffer_handle_bridge() {
    compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/input_runtime_demo",
    );
}

#[test]
fn compiles_terminal_io_demo_project_with_host_buffer_handle_bridge() {
    compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/terminal_io_demo",
    );
}

#[test]
fn compiles_inline_float_arithmetic_with_cpu_float_consts() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn add32() -> f32 {
            let sum: f32 = 1.5 + 2.25;
            return sum;
          }

          fn add64() -> f64 {
            return 1.5 + 2.25;
          }

          fn main() -> i64 {
            let lhs: f32 = add32();
            let rhs: f64 = add64();
            if lhs < 10.0 {
              return 1;
            }
            if rhs > 1.0 {
              return 2;
            }
            return 0;
          }
        }
        "#,
    )
    .expect("inline float arithmetic source should compile");

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "const_f32"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "const_f64"));
}

#[test]
fn compiles_float_tail_recursion_with_float_loop_llvm_lowering() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn walk(current: f64, acc: f64) -> f64 {
            if current <= 0.0 {
              return acc;
            }
            if current > 2.0 {
              return walk(current - 1.0, acc + current);
            }
            return walk(current - 1.0, acc * current);
          }

          fn main() -> i64 {
            let result: f64 = walk(4.0, 1.0);
            if result > 1.0 {
              return 7;
            }
            return 9;
          }
        }
        "#,
    )
    .expect("float tail recursion source should compile");

    assert!(artifacts.yir.nodes.iter().any(|node| {
        node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
    }));
    assert!(artifacts.llvm_ir.contains("fcmp ogt double"));
    assert!(artifacts.llvm_ir.contains("fsub double"));
    assert!(artifacts.llvm_ir.contains("fmul double"));
    assert!(artifacts.llvm_ir.contains("select i1"));
}
