use std::collections::BTreeMap;

use super::{cpu_param_binding, emit_cpu_function, emit_module, CpuCallScalarKind};
use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn assert_emit_module_error(module: &YirModule, expected_fragment: &str) {
    let error = emit_module(module).expect_err("LLVM lowering should fail");
    assert!(
        error.contains(expected_fragment),
        "expected error to contain `{expected_fragment}`, got `{error}`"
    );
}

#[test]
fn emits_dynamic_declare_for_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "arg0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["100000".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "sleep_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec!["c".to_owned(), "usleep".to_owned(), "arg0".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "arg0".to_owned(),
        to: "sleep_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @usleep(i64)"));
    assert!(llvm_ir.contains("call i64 @usleep(i64"));
}

#[test]
fn emits_dynamic_declare_for_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "arg0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["7".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "curve_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec![
                "c".to_owned(),
                "host_i32_curve".to_owned(),
                "arg0".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "arg0".to_owned(),
        to: "curve_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @host_i32_curve(i32)"));
    assert!(llvm_ir.contains("call i32 @host_i32_curve(i32"));
}

#[test]
fn emits_dynamic_declare_for_libc_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "usec".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["100000".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "sleep_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "usleep".to_owned(), "usec".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "usec".to_owned(),
        to: "sleep_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @usleep(i32)"));
    assert!(llvm_ir.contains("call i32 @usleep(i32"));
}

#[test]
fn emits_dynamic_declare_for_libc_close_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "fd".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["-1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "close_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "close".to_owned(), "fd".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "fd".to_owned(),
        to: "close_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @close(i32)"));
    assert!(llvm_ir.contains("call i32 @close(i32"));
}

#[test]
fn lowers_libc_puts_text_argument_as_ptr() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["hello libc".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "puts_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "puts".to_owned(), "message".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "message".to_owned(),
        to: "puts_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @puts(ptr)"));
    assert!(llvm_ir.contains("call i32 @puts(ptr"));
    assert!(!llvm_ir.contains("call i32 @puts(i32"));
}

#[test]
fn lowers_libc_strlen_text_argument_as_ptr() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["nuis".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "strlen_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec!["libc".to_owned(), "strlen".to_owned(), "message".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "message".to_owned(),
        to: "strlen_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @strlen(ptr)"));
    assert!(llvm_ir.contains("call i64 @strlen(ptr"));
    assert!(!llvm_ir.contains("call i64 @strlen(i64"));
}

#[test]
fn lowers_libc_write_mixed_arguments_with_text_as_ptr() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "fd".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["hello write".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "len".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["11".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "write_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "libc".to_owned(),
                "write".to_owned(),
                "fd".to_owned(),
                "message".to_owned(),
                "len".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["fd", "message", "len"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "write_call".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @write(i32, ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @write(i32"));
    assert!(llvm_ir.contains(", ptr "));
    assert!(!llvm_ir.contains("@write(i32, i64, i64)"));
}

#[test]
fn emits_dynamic_declare_for_libc_no_arg_i32_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "pid_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i32",
            vec!["libc".to_owned(), "getpid".to_owned()],
        )
        .unwrap(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i32 @getpid()"));
    assert!(llvm_ir.contains("call i32 @getpid()"));
}

#[test]
fn emits_dynamic_declare_for_text_ptr_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "message".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["hello".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "text_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "c".to_owned(),
                "host_accept_text_ptr".to_owned(),
                "message".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "message".to_owned(),
        to: "text_call".to_owned(),
    });

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @host_accept_text_ptr(ptr)"));
    assert!(llvm_ir.contains("call i64 @host_accept_text_ptr(ptr"));
}

#[test]
fn emits_dynamic_declare_for_buffer_ptr_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "len".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["8".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "fill".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "buffer".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.alloc_buffer",
            vec!["len".to_owned(), "fill".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "buffer_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "c".to_owned(),
                "host_fill_buffer_ptr".to_owned(),
                "buffer".to_owned(),
                "len".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("len", "buffer"),
        ("fill", "buffer"),
        ("buffer", "buffer_call"),
        ("len", "buffer_call"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @host_fill_buffer_ptr(ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @host_fill_buffer_ptr(ptr"));
}

#[test]
fn emits_module_with_contract_metadata_nodes_on_cpu_without_fake_cycles() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "seed".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["7".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "print_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.print", vec!["seed".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "lowering_cpu_target_config".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.target_config",
            vec![
                "arm64".to_owned(),
                "cpu.arm64.apple_aapcs64".to_owned(),
                "128".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "lowering_cpu_target_contract_type".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.text",
            vec![
                "arch=symbol:arm64;abi=symbol:cpu.arm64.apple_aapcs64;vector_bits=i64:128"
                    .to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "project_abi_cpu_selection_entry".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.text",
            vec!["mode=symbol:auto;abi=symbol:cpu.arm64.apple_aapcs64;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:llvm".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "project_abi_cpu_selection_summary_type".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.text",
            vec!["mode=symbol:auto;abi=symbol:cpu.arm64.apple_aapcs64;arch=symbol:arm64;os=symbol:darwin;object=symbol:mach-o;calling=symbol:aapcs64-darwin;backend=symbol:llvm".to_owned()],
        )
        .unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "seed".to_owned(),
        to: "print_0".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "lowering_cpu_target_contract_type".to_owned(),
        to: "lowering_cpu_target_config".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "project_abi_cpu_selection_summary_type".to_owned(),
        to: "project_abi_cpu_selection_entry".to_owned(),
    });
    for name in [
        "lowering_cpu_target_config",
        "lowering_cpu_target_contract_type",
        "project_abi_cpu_selection_entry",
        "project_abi_cpu_selection_summary_type",
    ] {
        module
            .node_lanes
            .insert(name.to_owned(), "contract".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.target_config"));
}

#[test]
fn lowers_cpu_select_between_structs_then_field_access() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("enabled", "true"),
        ("then_score", "42"),
        ("then_aux", "7"),
        ("else_score", "-1"),
        ("else_aux", "3"),
    ] {
        let instruction = if value == "true" {
            "cpu.const_bool"
        } else {
            "cpu.const_i64"
        };
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "then_pair".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec![
                "Pair".to_owned(),
                "score=then_score".to_owned(),
                "aux=then_aux".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "else_pair".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec![
                "Pair".to_owned(),
                "score=else_score".to_owned(),
                "aux=else_aux".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "then_pair".to_owned(),
                "else_pair".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_score".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.field", vec!["selected".to_owned(), "score".to_owned()]).unwrap(),
    });
    for (from, to) in [
        ("then_score", "then_pair"),
        ("then_aux", "then_pair"),
        ("else_score", "else_pair"),
        ("else_aux", "else_pair"),
        ("enabled", "selected"),
        ("then_pair", "selected"),
        ("else_pair", "selected"),
        ("selected", "selected_score"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.field `selected_score`"));
}

#[test]
fn lowers_cpu_select_between_bool_and_i64_condition_values_as_bool() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("enabled", "cpu.const_bool", "true"),
        ("explicit_false", "cpu.const_bool", "false"),
        ("lhs", "cpu.const_i64", "8"),
        ("rhs", "cpu.const_i64", "3"),
        ("ok_value", "cpu.const_i64", "0"),
        ("err_value", "cpu.const_i64", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "computed_bool_i64".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.gt", vec!["lhs".to_owned(), "rhs".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_bool".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "explicit_false".to_owned(),
                "computed_bool_i64".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "status".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "selected_bool".to_owned(),
                "ok_value".to_owned(),
                "err_value".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("lhs", "computed_bool_i64"),
        ("rhs", "computed_bool_i64"),
        ("enabled", "selected_bool"),
        ("explicit_false", "selected_bool"),
        ("computed_bool_i64", "selected_bool"),
        ("selected_bool", "status"),
        ("ok_value", "status"),
        ("err_value", "status"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected_bool`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `status`"));
}

#[test]
fn lowers_cpu_select_between_enum_variants_as_tagged_union() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("enabled", "cpu.const_bool", "true"),
        ("ok_payload", "cpu.const_i64", "7"),
        ("err_payload", "cpu.const_i64", "99"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "ok_variant".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Ok".to_owned(), "value=ok_payload".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "err_variant".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.struct",
            vec!["Result.Err".to_owned(), "value=err_payload".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.select",
            vec![
                "enabled".to_owned(),
                "ok_variant".to_owned(),
                "err_variant".to_owned(),
            ],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_is_err".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.variant_is",
            vec!["selected".to_owned(), "Result.Err".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "selected_ok_value".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.variant_field",
            vec![
                "selected".to_owned(),
                "Result.Ok".to_owned(),
                "value".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("ok_payload", "ok_variant"),
        ("err_payload", "err_variant"),
        ("enabled", "selected"),
        ("ok_variant", "selected"),
        ("err_variant", "selected"),
        ("selected", "selected_is_err"),
        ("selected", "selected_ok_value"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("select i1"));
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("ret i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select `selected`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_is `selected_is_err`"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.variant_field `selected_ok_value`"));
}

#[test]
fn emits_static_aot_tick_i64_values() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "tick".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.tick_i64", vec!["4".to_owned(), "3".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "bias".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["10".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.add", vec!["tick".to_owned(), "bias".to_owned()]).unwrap(),
    });
    for from in ["tick", "bias"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "sum".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("static AOT lowering freezes cpu.tick_i64"));
    assert!(llvm_ir.contains("add i64 4, 3"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.tick_i64"));
}

#[test]
fn emits_three_arg_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [("arg0", "1"), ("arg1", "2"), ("arg2", "3")] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "spawn_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "c".to_owned(),
                "host_subprocess_spawn".to_owned(),
                "arg0".to_owned(),
                "arg1".to_owned(),
                "arg2".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["arg0", "arg1", "arg2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "spawn_call".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @host_subprocess_spawn(i64, i64, i64)"));
    assert!(llvm_ir.contains("call i64 @host_subprocess_spawn(i64"));
}

#[test]
fn emits_helper_function_lanes_and_cpu_call_i64() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "seed".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["6".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "invoke".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.call_i64", vec!["inc".to_owned(), "seed".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "seed".to_owned(),
        to: "invoke".to_owned(),
    });

    module.nodes.push(Node {
        name: "inc_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "inc_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "inc_sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["inc_param_0".to_owned(), "inc_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "inc_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["inc_sum".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "inc_param_0".to_owned(),
        to: "inc_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "inc_one".to_owned(),
        to: "inc_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "inc_sum".to_owned(),
        to: "inc_ret".to_owned(),
    });
    for name in ["inc_param_0", "inc_one", "inc_sum", "inc_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:inc".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("define i64 @nuis_fn_inc(i64 %arg0)"));
    assert!(llvm_ir.contains("call i64 @nuis_fn_inc(i64"));
}

#[test]
fn emits_guard_return_as_real_branch() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "cond".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "early".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["64".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "guard".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.guard_return",
            vec!["cond".to_owned(), "early".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "later".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["7".to_owned()]).unwrap(),
    });
    for from in ["cond", "early"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "guard".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("br i1"));
    assert!(llvm_ir.contains("guard_return_then."));
    assert!(llvm_ir.contains("ret i64 %"));
    assert!(llvm_ir.contains("guard_return_cont."));
    assert_eq!(llvm_ir.matches("ret i64 ").count(), 2);
    assert!(llvm_ir.find("guard_return_cont.").unwrap() < llvm_ir.find("= add i64 0, 7").unwrap());
}

#[test]
fn emits_guard_print_return_as_real_branch() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "cond".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "msg".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["usage".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "early".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["64".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "guard".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.guard_print_return",
            vec!["cond".to_owned(), "msg".to_owned(), "early".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "later".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["7".to_owned()]).unwrap(),
    });
    for from in ["cond", "msg", "early"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "guard".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("br i1"));
    assert!(llvm_ir.contains("guard_print_return_then."));
    assert!(llvm_ir.contains("call i32 @puts(ptr"));
    assert_eq!(llvm_ir.matches("ret i64 ").count(), 2);
    assert!(llvm_ir.contains("guard_print_return_cont."));
}

#[test]
fn emits_branch_print_return_as_real_two_way_branch() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "cond".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "then_msg".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["ok".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "then_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "else_msg".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.text", vec!["fail".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "else_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["64".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "branch".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.branch_print_return",
            vec![
                "cond".to_owned(),
                "then_msg".to_owned(),
                "then_ret".to_owned(),
                "else_msg".to_owned(),
                "else_ret".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["cond", "then_msg", "then_ret", "else_msg", "else_ret"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "branch".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("branch_print_return_then."));
    assert!(llvm_ir.contains("branch_print_return_else."));
    assert!(llvm_ir.matches("call i32 @puts(ptr").count() >= 2);
    assert!(llvm_ir.matches("ret i64 ").count() >= 2);
}

#[test]
fn emits_i32_helper_returns_with_i32_ret_in_recursive_helpers() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    module.nodes.push(Node {
        name: "step_param".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i32", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i32", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "next".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.sub", vec!["step_param".to_owned(), "one".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i32", vec!["next".to_owned()]).unwrap(),
    });
    for from in ["step_param", "one", "next"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "step_ret".to_owned(),
        });
    }

    let ordered_node_names = vec![
        "step_param".to_owned(),
        "one".to_owned(),
        "next".to_owned(),
        "step_ret".to_owned(),
    ];
    let mut param_bindings = BTreeMap::new();
    param_bindings.insert(
        "step_param".to_owned(),
        cpu_param_binding(CpuCallScalarKind::I32, 0),
    );
    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<_, _>>();
    let mut global_counter = 0;

    let emitted = emit_cpu_function(
        &module,
        &resources,
        &ordered_node_names,
        &param_bindings,
        &BTreeMap::new(),
        CpuCallScalarKind::I32,
        &mut global_counter,
    )
    .expect("i32 helper lowering should succeed");

    assert!(emitted.body.contains("sub i32 "));
    assert!(emitted.body.contains("ret i32 "));
    assert!(!emitted.body.contains("ret i64 %"));
}

#[test]
fn emits_recursive_boolean_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("step", "1"),
        ("carry0", "0"),
        ("rhs0", "2"),
        ("rhs1", "6"),
        ("rhs2", "4"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "or".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "continue".to_owned(),
                "carry0".to_owned(),
                "current_eq".to_owned(),
                "rhs0".to_owned(),
                "keep".to_owned(),
                "keep".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "carry0", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp sgt i64"));
    assert!(llvm_ir.contains("icmp slt i64"));
    assert!(llvm_ir.contains("icmp ne i64"));
    assert!(llvm_ir.contains(" = or i1 "));
    assert!(llvm_ir.contains(" = and i1 "));
}

#[test]
fn emits_recursive_boolean_sync_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "10"),
        ("limit", "0"),
        ("step", "1"),
        ("carry0", "0"),
        ("rhs0", "8"),
        ("rhs1", "4"),
        ("rhs2", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "sub".to_owned(),
                "or".to_owned(),
                "current_eq".to_owned(),
                "rhs0".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "break".to_owned(),
                "carry0".to_owned(),
                "current_eq".to_owned(),
                "rhs0".to_owned(),
                "keep".to_owned(),
                "keep".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "carry0", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("icmp eq i64"));
    assert!(llvm_ir.contains("icmp sgt i64"));
    assert!(llvm_ir.contains("icmp ne i64"));
    assert!(llvm_ir.contains(" = and i1 "));
    assert!(llvm_ir.contains(" = or i1 "));
}

#[test]
fn emits_mixed_break_continue_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("step", "1"),
        ("rhs0", "2"),
        ("rhs1", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_continue".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_flow_action"));
    assert!(llvm_ir.contains("loop_flow_rhs"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_flow_or_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "12"),
        ("step", "1"),
        ("rhs0", "7"),
        ("rhs1", "2"),
        ("rhs2", "5"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
                "flow_break".to_owned(),
                "current_eq".to_owned(),
                "rhs2".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_flow_rhs") >= 2);
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_boolean_flow_or_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "16"),
        ("step", "1"),
        ("rhs0", "10"),
        ("rhs1", "3"),
        ("rhs2", "7"),
        ("rhs3", "1"),
        ("rhs4", "5"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "or".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "or".to_owned(),
                "current_eq".to_owned(),
                "rhs3".to_owned(),
                "current_gt".to_owned(),
                "rhs4".to_owned(),
                "flow_break".to_owned(),
                "current_lt".to_owned(),
                "rhs4".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in [
        "initial", "limit", "step", "rhs0", "rhs1", "rhs2", "rhs3", "rhs4",
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_flow_rhs") >= 2);
    assert!(count_occurrences(&llvm_ir, " = and i1 ") >= 1);
    assert!(count_occurrences(&llvm_ir, " = or i1 ") >= 2);
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_flow_cond_chain_cond"));
}

#[test]
fn emits_mixed_break_continue_sync_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "8"),
        ("limit", "0"),
        ("step", "1"),
        ("rhs0", "5"),
        ("rhs1", "2"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "sub".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_continue".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_post_flow_action"));
    assert!(llvm_ir.contains("loop_post_flow_rhs"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_post_flow_cond_chain_cond"));
}

#[test]
fn emits_mixed_break_continue_async_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("rhs0", "3"),
        ("rhs1", "2"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_continue".to_owned(),
                "current_lt".to_owned(),
                "rhs1".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_sum".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.add",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_sum".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_sum".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_sum".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_sum", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_async_flow_action"));
    assert!(llvm_ir.contains("loop_async_flow_rhs"));
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_flow_cond_chain_cond"));
}

#[test]
fn emits_mixed_break_continue_async_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "8"),
        ("limit", "0"),
        ("rhs0", "5"),
        ("rhs1", "2"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_continue".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_diff".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.sub",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_diff".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_diff".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_diff", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("loop_async_post_flow_action"));
    assert!(llvm_ir.contains("loop_async_post_flow_rhs"));
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_flow_or_async_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "10"),
        ("limit", "0"),
        ("rhs0", "8"),
        ("rhs1", "4"),
        ("rhs2", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
                "flow_break".to_owned(),
                "current_eq".to_owned(),
                "rhs2".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1", "rhs2"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_diff".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.sub",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_diff".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_diff".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_diff", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_rhs") >= 2);
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_cond"));
}

#[test]
fn emits_nested_boolean_flow_or_async_post_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "12"),
        ("limit", "0"),
        ("rhs0", "9"),
        ("rhs1", "6"),
        ("rhs2", "3"),
        ("rhs3", "2"),
        ("rhs4", "1"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_async_post_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "gt".to_owned(),
                "flow_or".to_owned(),
                "flow_break".to_owned(),
                "or".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs1".to_owned(),
                "current_ne".to_owned(),
                "rhs2".to_owned(),
                "flow_or".to_owned(),
                "flow_continue".to_owned(),
                "and".to_owned(),
                "current_gt".to_owned(),
                "rhs3".to_owned(),
                "current_ne".to_owned(),
                "rhs4".to_owned(),
                "flow_break".to_owned(),
                "current_eq".to_owned(),
                "rhs4".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "rhs0", "rhs1", "rhs2", "rhs3", "rhs4"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    module.nodes.push(Node {
        name: "step_param_0".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_one".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.const_i64", vec!["1".to_owned()]).unwrap(),
    });
    module.nodes.push(Node {
        name: "step_diff".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.sub",
            vec!["step_param_0".to_owned(), "step_one".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "step_ret".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.return_i64", vec!["step_diff".to_owned()]).unwrap(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_param_0".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_one".to_owned(),
        to: "step_diff".to_owned(),
    });
    module.edges.push(Edge {
        kind: EdgeKind::Dep,
        from: "step_diff".to_owned(),
        to: "step_ret".to_owned(),
    });
    for name in ["step_param_0", "step_one", "step_diff", "step_ret"] {
        module
            .node_lanes
            .insert(name.to_owned(), "fn:step".to_owned());
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_action") >= 3);
    assert!(count_occurrences(&llvm_ir, "loop_async_post_flow_rhs") >= 2);
    assert!(count_occurrences(&llvm_ir, " = and i1 ") >= 2);
    assert!(count_occurrences(&llvm_ir, " = or i1 ") >= 1);
    assert!(llvm_ir.contains("call i64 @nuis_fn_step(i64"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_exit"));
    assert!(llvm_ir.contains("br label %loop_while_scalar_async_post_flow_cond_chain_cond"));
}

#[test]
fn rejects_flow_and_in_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("step", "1"),
        ("rhs0", "2"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "flow_and".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    assert_emit_module_error(&module, "is missing flow control action");
}

#[test]
fn rejects_missing_flow_action_in_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [
        ("initial", "0"),
        ("limit", "8"),
        ("step", "1"),
        ("rhs0", "2"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "current_gt".to_owned(),
                "rhs0".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step", "rhs0"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    assert_emit_module_error(&module, "is missing flow control action");
}

#[test]
fn rejects_missing_flow_rhs_in_sync_flow_cond_chain() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, value) in [("initial", "0"), ("limit", "8"), ("step", "1")] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse("cpu.const_i64", vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_cond_chain",
            vec![
                "initial".to_owned(),
                "limit".to_owned(),
                "step".to_owned(),
                "lt".to_owned(),
                "add".to_owned(),
                "flow_break".to_owned(),
                "current_gt".to_owned(),
            ],
        )
        .unwrap(),
    });
    for from in ["initial", "limit", "step"] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: "loop".to_owned(),
        });
    }

    assert_emit_module_error(&module, "is missing flow control rhs");
}

#[test]
fn emits_dynamic_declare_for_libc_read_ref_buffer_cpu_extern_calls() {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    });
    for (name, instruction, value) in [
        ("fd", "cpu.const_i32", "-1"),
        ("len", "cpu.const_i64", "8"),
        ("fill", "cpu.const_i64", "0"),
    ] {
        module.nodes.push(Node {
            name: name.to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, vec![value.to_owned()]).unwrap(),
        });
    }
    module.nodes.push(Node {
        name: "scratch".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.alloc_buffer",
            vec!["len".to_owned(), "fill".to_owned()],
        )
        .unwrap(),
    });
    module.nodes.push(Node {
        name: "read_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.extern_call_i64",
            vec![
                "libc".to_owned(),
                "read".to_owned(),
                "fd".to_owned(),
                "scratch".to_owned(),
                "len".to_owned(),
            ],
        )
        .unwrap(),
    });
    for (from, to) in [
        ("len", "scratch"),
        ("fill", "scratch"),
        ("fd", "read_call"),
        ("scratch", "read_call"),
        ("len", "read_call"),
    ] {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }

    let llvm_ir = emit_module(&module).expect("LLVM lowering should succeed");
    assert!(llvm_ir.contains("declare i64 @read(i32, ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @read(i32"));
}
