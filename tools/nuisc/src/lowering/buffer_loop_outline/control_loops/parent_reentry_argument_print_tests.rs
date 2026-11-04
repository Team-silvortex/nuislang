use super::*;

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
