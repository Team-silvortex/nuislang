use super::*;
use division_execution::{execute_source, Case, Shape};

const ORIGINAL: &str = "if !enabled { return Carries { carry0: seed, carry1: seed }; }\n             return Carries { carry0: leaf(a, b), carry1: seed };";

const BODIES: &[(&str, Shape)] = &[
    (
        "if enabled { return Carries { carry0: leaf(a, b), carry1: seed }; }
      return Carries { carry0: seed, carry1: seed };",
        Shape::Aggregate,
    ),
    (
        "if enabled { return pack(leaf(a, b), seed); }
      else { return pack(seed, seed); }",
        Shape::Aggregate,
    ),
    (
        "if enabled {
        if a < 0 { return pack(leaf(a, b), seed); }
        return Carries { carry0: leaf(a, b), carry1: seed };
      } return pack(seed, seed);",
        Shape::Aggregate,
    ),
    (
        "let saved: Carries = pack(seed, seed);
      if enabled { return pack(leaf(a, b), saved.carry1); }
      return saved;",
        Shape::Aggregate,
    ),
    (
        "if enabled { let discarded: Carries = pack(leaf(a, b), seed); }
      return pack(seed, seed);",
        Shape::Discarded,
    ),
    (
        "if enabled { return pack(ignore(leaf(a, b)), seed); }
      return pack(seed, seed);",
        Shape::Argument,
    ),
];
const PREFIX: &str = "let saved: Carries = pack(leaf(a, b), seed);
    if !enabled { return pack(seed, seed); } return saved;";

#[test]
fn fallible_aggregate_returns_compose_with_typed_lifecycle_and_loop_exits() {
    assert_native_parity(include_str!("aggregate_division_loops.ns"), true);
}

fn source(op: &str, body: &str) -> String {
    division_execution::source(op, Shape::Aggregate)
        .replace(ORIGINAL, body)
        .replace(
            "fn main()",
            "@noinline fn pack(value: i64, seed: i64) -> Carries {
            return Carries { carry0: value, carry1: seed };
        } fn main()",
        )
}

#[test]
fn counted_loop_bearing_aggregate_branches_gain_guarded_lowering() {
    for op in ["/", "%"] {
        let source = source(op, BODIES[0].0).replace(
            &format!("return a {op} b;"),
            &format!(
                "let index: i64 = 0;
                while index < a {{ let index: i64 = index + 1; }}
                return (a + index) {op} b;"
            ),
        );
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let bridge = emit_registered(&compiled.yir, "counter").unwrap();
        assert!(bridge.llvm_ir.contains("native_loop_preflight"));
        assert!(bridge.llvm_ir.contains("integer_divisor_invalid"));
        assert!(compiled
            .yir
            .nodes
            .iter()
            .any(|node| node.op.instruction == "guard_return"));
    }
}

fn cases() -> Vec<Case> {
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -17,
        -3,
        -1,
        0,
        1,
        3,
        17,
        i64::MAX - 1,
        i64::MAX,
    ];
    let mut cases = Vec::new();
    for enabled in [false, true] {
        for left in values {
            for right in values {
                if !enabled || (right != 0 && (left, right) != (i64::MIN, -1)) {
                    cases.push(Case {
                        enabled,
                        left,
                        right,
                    });
                }
            }
        }
    }
    assert_eq!(cases.len(), 230);
    cases
}

#[test]
fn fallible_flat_branches_match_oracles_and_release_real_aggregate_temporaries() {
    for op in ["/", "%"] {
        for (body, shape) in BODIES {
            execute_source(op, *shape, &cases(), false, &source(op, body));
        }
    }
}

#[test]
fn fallible_flat_branches_trap_only_on_reached_operands_including_prefixes() {
    for op in ["/", "%"] {
        for (body, shape) in BODIES.iter().copied().chain([(PREFIX, Shape::Prefix)]) {
            for (left, right) in [(7, 0), (i64::MIN, -1)] {
                execute_source(
                    op,
                    shape,
                    &[Case {
                        enabled: !matches!(shape, Shape::Prefix),
                        left,
                        right,
                    }],
                    true,
                    &source(op, body),
                );
            }
        }
    }
}

#[test]
fn flat_branch_literal_failures_stay_behind_the_selected_guard() {
    for op in ["/", "%"] {
        for (left, right, lhs, rhs) in [
            (7, 0, "7", "0"),
            (i64::MIN, -1, "(0 - 9223372036854775807 - 1)", "(0 - 1)"),
        ] {
            let source = source(op, BODIES[1].0).replace(
                &format!("return a {op} b;"),
                &format!("return {lhs} {op} {rhs};"),
            );
            for enabled in [false, true] {
                execute_source(
                    op,
                    Shape::Aggregate,
                    &[Case {
                        enabled,
                        left,
                        right,
                    }],
                    enabled,
                    &source,
                );
            }
        }
    }
}

#[test]
fn registered_flat_state_branches_keep_declared_field_names_and_declaration_order_parity() {
    let publish = "let next: Carries = compute(enabled, a, b, seed);\n             return State { value: next.carry0, seed: next.carry1 };";
    for op in ["/", "%"] {
        let body = BODIES[0]
            .0
            .replace("Carries", "State")
            .replace("carry0", "value")
            .replace("carry1", "seed");
        let source = division_execution::source(op, Shape::Aggregate).replace(publish, &body);
        execute_source(op, Shape::Aggregate, &cases(), false, &source);
        for (left, right) in [(7, 0), (i64::MIN, -1)] {
            execute_source(
                op,
                Shape::Aggregate,
                &[Case {
                    enabled: true,
                    left,
                    right,
                }],
                true,
                &source,
            );
        }
    }
}
