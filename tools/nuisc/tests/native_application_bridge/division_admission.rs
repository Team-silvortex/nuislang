use super::*;
use helpers::{edge, push_node};

const DIVISION: &str = include_str!("division_loops.ns");

#[test]
fn division_remainder_compose_with_multi_state_loop_exits_and_typed_lifecycle() {
    let project = Project::with_source(DIVISION);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    for instruction in ["div", "rem", "call_owned_struct"] {
        assert!(module
            .nodes
            .iter()
            .any(|node| node.op.instruction == instruction));
    }
    let bridge = emit_registered(&module, "counter").unwrap();
    for opcode in [
        "sdiv i64",
        "srem i64",
        "integer_divisor_invalid",
        "loop_break_control_invalid",
    ] {
        assert!(bridge.llvm_ir.contains(opcode), "{opcode}");
    }
    assert_native_parity(DIVISION, true);
}

#[test]
fn generic_division_and_remainder_require_two_exact_i64_operands() {
    for op in ["/", "%"] {
        let project = Project::with_source(&division_execution::source(
            op,
            division_execution::Shape::Scalar,
        ));
        let base = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        emit_registered(&base, "counter").unwrap();
        let instruction = if op == "/" { "div" } else { "rem" };
        let operation = base
            .nodes
            .iter()
            .find(|node| node.op.instruction == instruction)
            .unwrap();
        for kind in ["bool", "i32", "f32", "f64"] {
            for slots in [&[0_usize][..], &[1][..], &[0, 1][..]] {
                let mut drift = base.clone();
                let replacement = "invalid_divisor_kind";
                push_node(
                    &mut drift,
                    "leaf",
                    replacement,
                    &format!("const_{kind}"),
                    vec![if kind == "bool" { "true" } else { "1" }.to_owned()],
                );
                drift
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "leaf")
                    .unwrap()
                    .body_nodes
                    .push(replacement.to_owned());
                let node = drift
                    .nodes
                    .iter_mut()
                    .find(|node| node.name == operation.name)
                    .unwrap();
                for slot in slots {
                    node.op.args[*slot] = replacement.to_owned();
                }
                edge(&mut drift, replacement, &operation.name);
                let error = emit_registered(&drift, "counter").unwrap_err();
                assert!(
                    error.contains(&operation.name) && error.contains("declared scalar kind"),
                    "{error}"
                );
            }
        }
        for count in [0, 1, 3] {
            let mut drift = base.clone();
            let node = drift
                .nodes
                .iter_mut()
                .find(|node| node.name == operation.name)
                .unwrap();
            node.op.args.resize(count, operation.op.args[0].clone());
            assert!(emit_registered(&drift, "counter").is_err());
        }
        for typed in ["div_i32", "div_f32", "div_f64"] {
            let mut drift = base.clone();
            drift
                .nodes
                .iter_mut()
                .find(|node| node.name == operation.name)
                .unwrap()
                .op
                .instruction = typed.to_owned();
            let error = emit_registered(&drift, "counter").unwrap_err();
            assert!(
                error.contains(&format!("does not admit cpu.{typed}")),
                "{error}"
            );
        }
    }
}

#[test]
fn fallible_aggregate_returns_use_guarded_helpers_instead_of_speculation() {
    for op in ["/", "%"] {
        for body in [
            "if enabled { return Carries { carry0: leaf(a, b), carry1: seed }; }
             return Carries { carry0: seed, carry1: seed };",
            "if enabled { return Carries { carry0: leaf(a, b), carry1: seed }; }
             else { return Carries { carry0: seed, carry1: seed }; }",
            "if enabled { return pack(leaf(a, b), seed); }
             else { return pack(seed, seed); }",
        ] {
            let source = division_execution::source(op, division_execution::Shape::Aggregate)
                .replace(
                    "if !enabled { return Carries { carry0: seed, carry1: seed }; }\n             return Carries { carry0: leaf(a, b), carry1: seed };",
                    body,
                )
                .replace(
                    "fn main()",
                    "@noinline fn pack(a: i64, b: i64) -> Carries {
                        return Carries { carry0: a, carry1: b };
                    } fn main()",
                );
            let project = Project::with_source(&source);
            let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
            emit_registered(&module, "counter").unwrap();
            assert!(
                module
                    .nodes
                    .iter()
                    .any(|n| n.op.instruction == "guard_return"),
                "{body}"
            );
        }
    }
}
