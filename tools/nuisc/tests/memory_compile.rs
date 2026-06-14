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

#[test]
fn compiles_text_serialization_intrinsics_into_buffer() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let text_bytes: i64 = serialize_text_into("ok", buffer, 1);
            let digits: i64 = serialize_i64_into(42, buffer, 4);
            return
              text_len("ok")
              + text_bytes
              + digits
              + load_at(buffer, 1)
              + load_at(buffer, 4);
          }
        }
        "#,
    )
    .expect("text serialization intrinsics source should compile");

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.body.iter().any(|stmt| match stmt {
            NirStmt::Let { value, .. } | NirStmt::Return(Some(value)) => {
                contains_serialize_intrinsic(value)
            }
            _ => false,
        })
    }));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_serialize_text_into"));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_serialize_i64_into"));
    assert!(artifacts.llvm_ir.contains("ptrtoint ptr"));
}

#[test]
fn compiles_scalar_serialization_roundtrip_intrinsics() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let bool_bytes: i64 = serialize_bool_into(true, buffer, 0);
            let digits: i64 = serialize_i64_into(314, buffer, 6);
            let parsed: i64 = deserialize_i64_from(buffer, 6, digits);
            return
              bool_bytes
              + digits
              + parsed
              + load_at(buffer, 0)
              + load_at(buffer, 6);
          }
        }
        "#,
    )
    .expect("scalar serialization roundtrip source should compile");

    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_serialize_bool_into"));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_deserialize_i64_from"));
}

#[test]
fn compiles_text_deserialization_intrinsics() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let written: i64 = serialize_text_into("hello", buffer, 2);
            let restored: String = deserialize_text_from(buffer, 2, written);
            let parsed_flag: bool = deserialize_bool_from(buffer, 0, 1);
            return text_len(restored) + if parsed_flag { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("text deserialization source should compile");

    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_deserialize_text_from"));
    assert!(artifacts.llvm_ir.contains("call ptr @nuis_host_text_ptr"));
}

#[test]
fn compiles_text_slice_predicate_intrinsics() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let written: i64 = serialize_text_into("hello-world", buffer, 3);
            let exact: bool = deserialize_text_equals(buffer, 3, written, "hello-world");
            let prefix: bool = deserialize_text_starts_with(buffer, 3, written, "hello");
            return if exact && prefix { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("text slice predicate intrinsics source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_deserialize_text_equals"
    ));
    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_deserialize_text_starts_with"
    ));
}

#[test]
fn compiles_buffer_search_and_text_contains_intrinsics() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(64, 0);
            let written: i64 = serialize_text_into("header:value", buffer, 5);
            let colon: i64 = buffer_find_byte(buffer, 5, written, 58);
            let value_at: i64 = buffer_find_text(buffer, 5, written, "value");
            let has_value: bool = deserialize_text_contains(buffer, 5, written, "value");
            return if colon > 0 && value_at > colon && has_value { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("buffer search and text contains intrinsics source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_buffer_find_byte"
    ));
    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_deserialize_text_contains"
    ));
    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_buffer_find_text"
    ));
}

#[test]
fn compiles_text_slice_suffix_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(64, 0);
            let written: i64 = serialize_text_into("content-type: text/plain", buffer, 0);
            let suffix: bool = deserialize_text_ends_with(buffer, 0, written, "plain");
            return if suffix { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("text slice suffix intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_deserialize_text_ends_with"
    ));
}

#[test]
fn compiles_buffer_line_end_search_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(128, 0);
            let written: i64 = serialize_text_into("Host: example.com\r\nAccept: */*\n", buffer, 0);
            let line_end: i64 = buffer_find_line_end(buffer, 0, written);
            let colon: i64 = buffer_find_byte(buffer, 0, line_end, 58);
            return if line_end > colon && colon > 0 { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("buffer line end search intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_buffer_find_line_end"
    ));
    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_buffer_find_byte"
    ));
}

#[test]
fn compiles_buffer_trim_line_end_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(128, 0);
            let written: i64 = serialize_text_into("Host: example.com\r\n", buffer, 0);
            let trimmed: i64 = buffer_trim_line_end(buffer, 0, written);
            let header: String = deserialize_text_from(buffer, 0, trimmed);
            return if text_len(header) == trimmed { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("buffer trim line end intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_buffer_trim_line_end"
    ));
    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_deserialize_text_from"
    ));
}

#[test]
fn compiles_parse_header_line_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(128, 0);
            let written: i64 = serialize_text_into("Content-Type: text/plain\r\n", buffer, 0);
            let value: String = parse_header_line(buffer, 0, written, "Content-Type");
            return if text_len(value) > 0 { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("parse header line intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_parse_header_line"
    ));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_parse_header_line"));
}

#[test]
fn compiles_find_header_value_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(256, 0);
            let written: i64 = serialize_text_into("Host: example.com\r\nContent-Type: text/plain\r\n\r\n", buffer, 0);
            let value: String = find_header_value(buffer, 0, written, "Content-Type");
            return if text_len(value) > 0 { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("find header value intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_find_header_value"
    ));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_find_header_value"));
}

#[test]
fn compiles_find_status_line_reason_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(256, 0);
            let written: i64 = serialize_text_into("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n", buffer, 0);
            let reason: String = find_status_line_reason(buffer, 0, written);
            return if text_len(reason) > 0 { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("find status line reason intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_find_status_line_reason"
    ));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_find_status_line_reason"));
}

#[test]
fn compiles_parse_http_response_summary_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(512, 0);
            let written: i64 = serialize_text_into("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 12\r\n\r\n", buffer, 0);
            let summary: String = parse_http_response_summary(buffer, 0, written);
            return if text_len(summary) > 0 { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("parse http response summary intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_parse_http_response_summary"
    ));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_parse_http_response_summary"));
}

#[test]
fn compiles_parse_http_request_summary_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(512, 0);
            let written: i64 = serialize_text_into("GET /hello HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n", buffer, 0);
            let summary: String = parse_http_request_summary(buffer, 0, written);
            return if text_len(summary) > 0 { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("parse http request summary intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_parse_http_request_summary"
    ));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_parse_http_request_summary"));
}

#[test]
fn compiles_parse_http_roundtrip_summary_intrinsic() {
    let artifacts = nuisc::pipeline::compile_source(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let request: ref Buffer = alloc_buffer(512, 0);
            let response: ref Buffer = alloc_buffer(512, 0);
            let request_len: i64 = serialize_text_into("GET /hello HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n", request, 0);
            let response_len: i64 = serialize_text_into("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 12\r\n\r\n", response, 0);
            let summary: String = parse_http_roundtrip_summary(request, 0, request_len, response, 0, response_len);
            return if text_len(summary) > 0 { 1 } else { 0 };
          }
        }
        "#,
    )
    .expect("parse http roundtrip summary intrinsic source should compile");

    assert!(nir_contains_host_callee(
        &artifacts.nir,
        "host_parse_http_roundtrip_summary"
    ));
    assert!(artifacts
        .llvm_ir
        .contains("call i64 @host_parse_http_roundtrip_summary"));
}

fn contains_serialize_intrinsic(expr: &NirExpr) -> bool {
    match expr {
        NirExpr::CpuExternCall { callee, .. } => {
            callee == "host_text_len"
                || callee == "host_serialize_text_into"
                || callee == "host_serialize_i64_into"
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            contains_serialize_intrinsic(lhs) || contains_serialize_intrinsic(rhs)
        }
        _ => false,
    }
}

fn nir_contains_host_callee(module: &nuis_semantics::model::NirModule, callee: &str) -> bool {
    module
        .functions
        .iter()
        .flat_map(|function| function.body.iter())
        .any(|stmt| stmt_contains_host_callee(stmt, callee))
}

fn stmt_contains_host_callee(stmt: &NirStmt, callee: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => expr_contains_host_callee(value, callee),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_host_callee(condition, callee)
                || then_body
                    .iter()
                    .any(|stmt| stmt_contains_host_callee(stmt, callee))
                || else_body
                    .iter()
                    .any(|stmt| stmt_contains_host_callee(stmt, callee))
        }
        NirStmt::While { condition, body } => {
            expr_contains_host_callee(condition, callee)
                || body
                    .iter()
                    .any(|stmt| stmt_contains_host_callee(stmt, callee))
        }
        NirStmt::Return(Some(value)) => expr_contains_host_callee(value, callee),
        NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => false,
    }
}

fn expr_contains_host_callee(expr: &NirExpr, callee: &str) -> bool {
    match expr {
        NirExpr::CpuExternCall { callee: found, .. } => found == callee,
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_contains_host_callee(lhs, callee) || expr_contains_host_callee(rhs, callee)
        }
        _ => false,
    }
}
