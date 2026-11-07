use super::*;

#[path = "state_control_exit_argument_print_tests.rs"]
mod state_control_exit;
#[path = "state_division_exit_argument_print_tests.rs"]
mod state_division_exits;
#[path = "state_helper_exit_argument_print_tests.rs"]
mod state_helper_exits;
#[path = "state_parameter_exit_argument_print_tests.rs"]
mod state_parameter_exits;
#[path = "state_record_parameter_exit_argument_print_tests.rs"]
mod state_record_parameters;

fn admitted(source: &str, reversed: bool) -> bool {
    let mut module = crate::frontend::parse_nuis_module(source).unwrap();
    if reversed {
        module.functions.reverse();
    }
    let layouts = control_values::layouts(&module);
    scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("work")
}

#[test]
fn parent_reentry_argument_print_accepts_dynamic_parent_entries() {
    accepts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/parent_reentry_argument_print.ns"
    )));
}

#[test]
fn child_progress_argument_print_accepts_ready_child_induction() {
    accepts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/child_progress_argument_print.ns"
    )));
}

#[test]
fn parent_bound_argument_print_accepts_ready_per_parent_counts() {
    accepts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/parent_bound_argument_print.ns"
    )));
}

#[test]
fn fallible_bound_argument_print_accepts_reached_producer_checks() {
    accepts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/fallible_bound_argument_print.ns"
    )));
}

#[test]
fn state_bound_argument_print_accepts_current_ready_carry() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_bound_argument_print.ns"
    ));
    for op in ["<", "<=", ">", ">=", "==", "!="] {
        accepts(&source.replace("total < 0.0", &format!("total {op} 0.0")));
    }
}

#[test]
fn state_break_argument_print_accepts_post_update_break_and_observed_index() {
    accepts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_break_argument_print.ns"
    )));
}

#[test]
fn state_break_argument_print_keeps_suffix_owner_and_bound_refusals() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_break_argument_print.ns"
    ));
    refuses(source);
    let exit = "if child == cut { print(78); break; }";
    for bad in [
        source.replace("print(71);", "let __nuis_buffer_break: i64 = 0; print(71);"),
        source.replace("break;", "break; print(81);"),
        source
            .replace("let child_trips =", "let mut child_trips =")
            .replace("print(74);", "child_trips = child_trips + 1; print(74);"),
        source
            .replace(exit, "")
            .replace("print(71);", &format!("{exit} print(71);")),
    ] {
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(!admitted(&bad.replace("f64", kind), reversed));
            }
        }
    }
}

#[test]
fn state_bound_argument_print_ready_float_comparisons_preserve_types_and_epochs() {
    for kind in ["f32", "f64"] {
        let scalar = scalar_type(kind);
        let layouts = control_values::CarryLayouts::from([(
            "State".into(),
            vec![("total".into(), scalar.clone())],
        )]);
        let catalog = ScalarHelpers::new();
        let mut locals = Locals {
            scope: Scope::from([
                ("live".into(), scalar.clone()),
                ("other".into(), scalar.clone()),
                ("record".into(), scalar_type("State")),
            ]),
            writable: BTreeSet::from(["live".into(), "other".into(), "record".into()]),
            available: BTreeSet::new(),
        };
        let updates = locals.writable.clone();
        let zero = if kind == "f32" {
            NirExpr::F32("0.0".into())
        } else {
            NirExpr::F64("0.0".into())
        };
        let compare = |lhs, op, rhs| NirExpr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
        let check = |value: &NirExpr, locals: &Locals, own| {
            expression(value, locals, &updates, own, &catalog, &layouts, 0)
        };
        for op in [
            NirBinaryOp::Eq,
            NirBinaryOp::Ne,
            NirBinaryOp::Lt,
            NirBinaryOp::Le,
            NirBinaryOp::Gt,
            NirBinaryOp::Ge,
        ] {
            let live = compare(NirExpr::Var("live".into()), op, zero.clone());
            let other = compare(NirExpr::Var("other".into()), op, zero.clone());
            let field = compare(
                NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var("record".into())),
                    field: "total".into(),
                },
                op,
                zero.clone(),
            );
            locals.available.clear();
            assert!(check(&live, &locals, None).is_none());
            assert_eq!(
                check(&live, &locals, Some("live")),
                Some(scalar_type("bool"))
            );
            assert!(check(&other, &locals, Some("live")).is_none());
            assert!(check(&field, &locals, None).is_none());
            locals.available.extend(["live".into(), "record".into()]);
            assert_eq!(check(&live, &locals, None), Some(scalar_type("bool")));
            assert_eq!(check(&field, &locals, None), Some(scalar_type("bool")));
            let opposite = if kind == "f32" {
                NirExpr::F64("0.0".into())
            } else {
                NirExpr::F32("0.0".into())
            };
            for bad in [opposite, NirExpr::Int(0), NirExpr::Bool(false)] {
                assert!(check(
                    &compare(NirExpr::Var("live".into()), op, bad),
                    &locals,
                    None
                )
                .is_none());
            }
        }
        for op in [NirBinaryOp::Rem, NirBinaryOp::Xor] {
            assert!(check(&compare(zero.clone(), op, zero.clone()), &locals, None).is_none());
        }
    }
}

#[test]
fn state_bound_argument_print_keeps_scope_and_mutable_bound_refusals() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_bound_argument_print.ns"
    ));
    refuses(source);
    let changed = source
        .replace("let child_trips =", "let mut child_trips =")
        .replace("print(74);", "child_trips = child_trips + 1; print(74);");
    for kind in ["f32", "f64"] {
        for reversed in [false, true] {
            assert!(!admitted(&changed.replace("f64", kind), reversed));
        }
    }
}

#[test]
fn fallible_bound_argument_print_rejects_unready_and_mutated_bounds() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/fallible_bound_argument_print.ns"
    ));
    refuses(source);
    let changed = source
        .replace("let child_trips =", "let mut child_trips =")
        .replace("print(74);", "child_trips = child_trips + 1; print(74);");
    for kind in ["f32", "f64"] {
        for reversed in [false, true] {
            assert!(!admitted(&changed.replace("f64", kind), reversed));
        }
    }
}

fn accepts(template: &str) {
    for kind in ["f32", "f64"] {
        let source = template.replace("f64", kind);
        for reversed in [false, true] {
            assert!(admitted(&source, reversed));
            for count in [0, 2, 3] {
                assert!(admitted(
                    &source.replace("outer < parents", &format!("outer < {count}")),
                    reversed
                ));
            }
        }
    }
}

#[test]
fn parent_reentry_argument_print_rejects_unready_and_escaping_entries() {
    refuses(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/parent_reentry_argument_print.ns"
    )));
}

#[test]
fn child_progress_argument_print_rejects_unready_and_escaping_entries() {
    refuses(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/child_progress_argument_print.ns"
    )));
}

#[test]
fn parent_bound_argument_print_rejects_scope_escape_and_child_mutated_bounds() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/parent_bound_argument_print.ns"
    ));
    refuses(source);
    let changed = source
        .replace("let child_trips =", "let mut child_trips =")
        .replace("print(74);", "child_trips = child_trips + 1; print(74);");
    for kind in ["f32", "f64"] {
        for reversed in [false, true] {
            assert!(!admitted(&changed.replace("f64", kind), reversed));
        }
    }
}

fn refuses(source: &str) {
    let init = "let mut current = Pair { total: total, code: left, flag: selected };";
    for kind in ["f32", "f64"] {
        let source = source.replace("f64", kind);
        let stale = source.replace(init, "").replace(
            "while outer < parents {",
            &format!("{init} while outer < parents {{"),
        );
        for bad in [
            stale,
            source.replace("print(74);", "let alias = epoch_word;"),
            source
                .replace(
                    "let epoch_word: i64 =",
                    "if selected { let epoch_word: i64 =",
                )
                .replace("print(74);", "} print(74);"),
        ] {
            for reversed in [false, true] {
                assert!(!admitted(&bad, reversed));
            }
        }
    }
}

#[test]
fn state_continue_argument_print_accepts_one_step_and_selected_suffix() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_continue_argument_print.ns"
    ));
    accepts(source);
    accepts(&source.replace("if child == cut {", "if child != cut {} else {"));
}

#[test]
fn state_continue_argument_print_rejects_missing_duplicate_and_moved_steps() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_continue_argument_print.ns"
    ));
    refuses(source);
    let step = "child = child + 1; continue;";
    let exit = "if child == cut { print(78); child = child + 1; continue; }";
    for bad in [
        source.replace(step, "continue;"),
        source.replace(step, "child = child + 2; continue;"),
        source.replace(step, "child = child + 1; child = child + 1; continue;"),
        source.replace(step, "child = child + 1; print(81); continue;"),
        source.replace(step, "continue; child = child + 1;"),
        source.replace(
            "print(71);",
            "let __nuis_buffer_continue: i64 = 1; print(71);",
        ),
        source
            .replace(exit, "")
            .replace("print(71);", &format!("{exit} print(71);")),
        source
            .replace("let child_trips =", "let mut child_trips =")
            .replace("print(74);", "child_trips = child_trips + 1; print(74);"),
    ] {
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(
                    !admitted(&bad.replace("f64", kind), reversed),
                    "unexpected admission: {bad}"
                );
            }
        }
    }
}

#[test]
fn state_mixed_argument_print_accepts_ordered_and_mirrored_exit_guards() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_mixed_argument_print.ns"
    ));
    accepts(source);
    accepts(
        &source
            .replace("if child == cut {", "if child != cut {} else {")
            .replace("if child == stop_at {", "if child != stop_at {} else {"),
    );
    let first = "if child == cut { print(78); child = child + 1; continue; }";
    let second = "if child == stop_at { print(81); break; }";
    accepts(&source.replace(
        &format!("{first}\n                {second}"),
        &format!("{second}\n                {first}"),
    ));
}

#[test]
fn state_mixed_argument_print_rejects_missing_duplicate_and_moved_steps() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_mixed_argument_print.ns"
    ));
    refuses(source);
    let step = "child = child + 1; continue;";
    let exit = "if child == stop_at { print(81); break; }";
    for bad in [
        source.replace(step, "continue;"),
        source.replace(step, "child = child + 2; continue;"),
        source.replace(step, "child = child + 1; child = child + 1; continue;"),
        source.replace(step, "child = child + 1; print(82); continue;"),
        source.replace(step, "continue; child = child + 1;"),
        source.replace(
            "print(71);",
            "let __nuis_buffer_continue: i64 = 1; print(71);",
        ),
        source.replace("print(71);", "let __nuis_buffer_break: i64 = 0; print(71);"),
        source.replace("break;", "break; print(82);"),
        source
            .replace(exit, "")
            .replace("print(71);", &format!("{exit} print(71);")),
        source
            .replace("let child_trips =", "let mut child_trips =")
            .replace("print(74);", "child_trips = child_trips + 1; print(74);"),
    ] {
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(
                    !admitted(&bad.replace("f64", kind), reversed),
                    "unexpected admission: {bad}"
                );
            }
        }
    }
}

#[test]
fn state_predicate_argument_print_accepts_ordered_checked_exits() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_predicate_argument_print.ns"
    ));
    accepts(source);
    let first = "if continue_at(child, cut, continue_guard - (outer * 16 + child)) { print(78); child = child + 1; continue; }";
    let second =
        "if break_at(child, stop_at, break_guard - (outer * 16 + child)) { print(81); break; }";
    accepts(&source.replace(
        &format!("{first}\n                {second}"),
        &format!("{second}\n                {first}"),
    ));
}

#[test]
fn state_predicate_argument_print_keeps_step_type_and_scope_refusals() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_predicate_argument_print.ns"
    ));
    refuses(source);
    let step = "child = child + 1; continue;";
    for bad in [
        source.replace(step, "continue;"),
        source.replace(step, "child = child + 2; continue;"),
        source.replace(step, "child = child + 1; child = child + 1; continue;"),
        source.replace(step, "child = child + 1; print(82); continue;"),
        source.replace("break;", "break; print(82);"),
        source.replace(
            "print(71);",
            "let __nuis_buffer_continue: i64 = 1; print(71);",
        ),
        source
            .replace("let child_trips =", "let mut child_trips =")
            .replace("print(74);", "child_trips = child_trips + 1; print(74);"),
    ] {
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(
                    !admitted(&bad.replace("f64", kind), reversed),
                    "unexpected admission: {bad}"
                );
            }
        }
    }
    for kind in ["f32", "f64"] {
        let typed = source.replace("f64", kind);
        for (bad, diagnostic) in [
            (
                typed.replace("continue_at(child, cut,", "continue_at(current.total, cut,"),
                "function `continue_at` argument 1 expects `i64`",
            ),
            (
                typed.replace("break_at(child, stop_at,", "break_at(child, current.flag,"),
                "function `break_at` argument 2 expects `i64`",
            ),
        ] {
            assert!(crate::frontend::parse_nuis_module(&bad)
                .unwrap_err()
                .contains(diagnostic));
        }
    }
}

#[test]
fn state_short_circuit_argument_print_accepts_ordered_checked_operands() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_short_circuit_argument_print.ns"
    ));
    accepts(source);
    for (left, op, right) in [
        (
            "continue_at(child, cut, continue_guard - (outer * 16 + child))",
            "&&",
            "confirm_continue(child, permit, confirm_guard - (outer * 16 + child))",
        ),
        (
            "break_at(child, stop_at, break_guard - (outer * 16 + child))",
            "||",
            "fallback_break(child, fallback_at, fallback_guard - (outer * 16 + child))",
        ),
    ] {
        accepts(&source.replace(
            &format!("{left} {op} {right}"),
            &format!("{right} {op} {left}"),
        ));
    }
}

#[test]
fn state_short_circuit_argument_print_keeps_step_type_and_scope_refusals() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_short_circuit_argument_print.ns"
    ));
    refuses(source);
    let step = "child = child + 1; continue;";
    for bad in [
        source.replace(step, "continue;"),
        source.replace(step, "child = child + 2; continue;"),
        source.replace(step, "child = child + 1; child = child + 1; continue;"),
        source.replace(step, "child = child + 1; print(82); continue;"),
        source.replace("break;", "break; print(82);"),
        source.replace(
            "print(71);",
            "let __nuis_buffer_continue: i64 = 1; print(71);",
        ),
        source
            .replace("let child_trips =", "let mut child_trips =")
            .replace("print(74);", "child_trips = child_trips + 1; print(74);"),
    ] {
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(
                    !admitted(&bad.replace("f64", kind), reversed),
                    "unexpected admission: {bad}"
                );
            }
        }
    }
    for kind in ["f32", "f64"] {
        let typed = source.replace("f64", kind);
        for (bad, diagnostic) in [
            (
                typed.replace(
                    "confirm_continue(child, permit,",
                    "confirm_continue(current.total, permit,",
                ),
                "function `confirm_continue` argument 1 expects `i64`",
            ),
            (
                typed.replace(
                    "fallback_break(child, fallback_at,",
                    "fallback_break(child, current.flag,",
                ),
                "function `fallback_break` argument 2 expects `i64`",
            ),
        ] {
            assert!(crate::frontend::parse_nuis_module(&bad)
                .unwrap_err()
                .contains(diagnostic));
        }
    }
}

#[test]
fn state_grouped_argument_print_accepts_explicit_logical_groups() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_grouped_argument_print.ns"
    ));
    accepts(source);
    for (normal, regrouped) in [
        ("continue_at(child, cut, continue_guard - (outer * 16 + child)) && (confirm_continue(child, permit, confirm_guard - (outer * 16 + child)) || rescue_continue(child, rescue, rescue_guard - (outer * 16 + child)))", "(continue_at(child, cut, continue_guard - (outer * 16 + child)) && confirm_continue(child, permit, confirm_guard - (outer * 16 + child))) || rescue_continue(child, rescue, rescue_guard - (outer * 16 + child))"),
        ("(break_at(child, stop_at, break_guard - (outer * 16 + child)) || fallback_break(child, fallback_at, fallback_guard - (outer * 16 + child))) && confirm_break(child, seal, seal_guard - (outer * 16 + child))", "break_at(child, stop_at, break_guard - (outer * 16 + child)) || (fallback_break(child, fallback_at, fallback_guard - (outer * 16 + child)) && confirm_break(child, seal, seal_guard - (outer * 16 + child)))"),
    ] {
        let changed = source.replace(normal, regrouped);
        assert_ne!(changed, source);
        accepts(&changed);
    }
}

#[test]
fn state_grouped_argument_print_keeps_step_type_and_scope_refusals() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_grouped_argument_print.ns"
    ));
    refuses(source);
    let step = "child = child + 1; continue;";
    for bad in [
        source.replace(step, "continue;"),
        source.replace(step, "child = child + 2; continue;"),
        source.replace(step, "child = child + 1; child = child + 1; continue;"),
        source.replace(step, "child = child + 1; print(82); continue;"),
        source.replace("break;", "break; print(82);"),
        source.replace(
            "print(71);",
            "let __nuis_buffer_continue: i64 = 1; print(71);",
        ),
        source
            .replace("let child_trips =", "let mut child_trips =")
            .replace("print(74);", "child_trips = child_trips + 1; print(74);"),
    ] {
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(
                    !admitted(&bad.replace("f64", kind), reversed),
                    "unexpected admission: {bad}"
                );
            }
        }
    }
    for kind in ["f32", "f64"] {
        let typed = source.replace("f64", kind);
        for (bad, diagnostic) in [
            (
                typed.replace(
                    "rescue_continue(child, rescue,",
                    "rescue_continue(current.total, rescue,",
                ),
                "function `rescue_continue` argument 1 expects `i64`",
            ),
            (
                typed.replace(
                    "confirm_break(child, seal,",
                    "confirm_break(child, current.flag,",
                ),
                "function `confirm_break` argument 2 expects `i64`",
            ),
        ] {
            assert!(crate::frontend::parse_nuis_module(&bad)
                .unwrap_err()
                .contains(diagnostic));
        }
    }
}

#[test]
fn state_field_exit_argument_print_accepts_updated_record_predicates() {
    accepts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_field_exit_argument_print.ns"
    )));
}

#[test]
fn state_field_exit_argument_print_keeps_type_and_epoch_refusals() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/state_field_exit_argument_print.ns"
    ));
    refuses(source);
    let premature_exits = ["if continue_at(", "if (break_at("].map(|prefix| {
        let exit = source
            .lines()
            .find(|line| line.trim_start().starts_with(prefix))
            .unwrap()
            .trim();
        let without_exit = source.replace(exit, "");
        without_exit.replace("print(74);", &format!("{exit} print(74);"))
    });
    for bad in [
        source.replace("child = child + 1; continue;", "continue;"),
        source.replace(
            "child = child + 1; continue;",
            "child = child + 2; continue;",
        ),
        source.replace(
            "child = child + 1; continue;",
            "child = child + 1; child = child + 1; continue;",
        ),
        source.replace(
            "child = child + 1; continue;",
            "child = child + 1; print(82); continue;",
        ),
        source.replace("break;", "break; print(82);"),
        source.replace(
            "print(71);",
            "let __nuis_buffer_continue: i64 = 1; print(71);",
        ),
        source
            .replace("let child_trips =", "let mut child_trips =")
            .replace("print(74);", "child_trips = child_trips + 1; print(74);"),
    ]
    .into_iter()
    .chain(premature_exits)
    {
        assert_ne!(bad, source);
        for kind in ["f32", "f64"] {
            for reversed in [false, true] {
                assert!(
                    !admitted(&bad.replace("f64", kind), reversed),
                    "unexpected admission: {bad}"
                );
            }
        }
    }
    for kind in ["f32", "f64"] {
        let typed = source.replace("f64", kind);
        for (name, bad_value) in [
            ("confirm_continue", "current.flag"),
            ("confirm_break", "current.code"),
        ] {
            let bad = typed.replace(
                &format!("{name}(current.total,"),
                &format!("{name}({bad_value},"),
            );
            assert_ne!(bad, typed);
            assert!(crate::frontend::parse_nuis_module(&bad)
                .unwrap_err()
                .contains(&format!("function `{name}` argument 1 expects `{kind}`")));
        }
    }
}
