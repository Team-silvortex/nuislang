use super::*;
use crate::frontend::parse_nuis_module;

fn source(effects: &str) -> String {
    carries_tests::SOURCE.replace(
        "let index: i64 = index + stride;\n      let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
        &format!("{effects} let index: i64 = index + stride;"),
    )
}

#[test]
fn trailing_value_loops_preserve_pre_step_effects_and_canonical_exits() {
    for (effects, breaks) in [
        ("let total = total + index;", 0),
        ("let total = total + index; let checksum = checksum * total;", 0),
        ("let before = index; let total = total + before;", 0),
        ("break;", 1),
        ("if index > 0 { break; }", 1),
        ("if index > 0 { let index = index + stride; continue; } let total = total + index;", 0),
        ("if index > initial { break; } else { let index = index + stride; continue; }", 1),
        ("let child = 0; while child < index { let total = total + child; if child == 1 { break; } let child = child + 1; } let checksum = checksum + child;", 1),
        ("let child = 0; while child < limit { let total = total + child; break; let child = child + 1; } break;", 2),
    ] {
        let text = source(effects);
        let mut module = parse_nuis_module(&text).unwrap();
        let layouts = control_values::layouts(&module);
        for _ in 0..2 {
            let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
            for name in ["walk", "wrap", "choose"] {
                assert!(catalog.get(name).is_some_and(|f| f.may_loop), "{name}: {effects}");
            }
            module.functions.reverse();
        }
        assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert_eq!(outlined.break_controls.len(), breaks, "{effects}");
        for f in &module.functions {
            let mut bindings = BTreeSet::new();
            branches::collect_bindings(&f.body, &mut bindings);
            assert!(!bindings.iter().any(|name| name.starts_with("__nuis_advanced_index")));
        }
        for (name, flag) in &outlined.break_controls {
            let f = module.functions.iter().find(|f| &f.name == name).unwrap();
            let Some(NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) = f.body.last() else {
                panic!("even a flag-only break needs canonical projected transport");
            };
            assert_eq!(fields.last().unwrap().1, NirExpr::Var(flag.clone()));
        }
    }
}

#[test]
fn trailing_value_loops_reject_unstepped_continue_and_hidden_mutations() {
    for effects in [
        "continue;",
        "if index > initial { continue; }",
        "if index > initial { let index = index + 1; continue; }",
        "if index > initial { let index = index + stride; let total = total + index; continue; }",
        "if index > initial { let index = index + stride; let index = index + stride; continue; }",
        "if index > initial { let index = index + stride; break; }",
        "if false { let limit = limit + 1; break; }",
        "if false { let stride = stride + 1; let index = index + stride; continue; }",
        "let total = total + checksum; let checksum = checksum + 1;",
        "if index > initial { print(index); break; }",
        "if index > initial { break; let total = total + index; }",
        "let child = 0; while child < index { let child = child + 1; let index = index + stride; }",
    ] {
        let module = parse_nuis_module(&source(effects)).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk"),
            "{effects}"
        );
    }
    for (from, to) in [
        ("let index: i64 = initial", "const index: i64 = initial"),
        ("let total: i64 = initial", "const total: i64 = initial"),
        ("index < limit", "index < index"),
        ("index + stride", "index + index"),
        ("index + stride", "index + (stride / limit)"),
    ] {
        let module =
            parse_nuis_module(&source("let total = total + index;").replace(from, to)).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk"),
            "{to}"
        );
    }
}

#[test]
fn trailing_value_exit_normalization_bounds_source_and_generated_guards() {
    let mut deep = "break;".to_owned();
    for _ in 0..33 {
        deep = format!("if index > initial {{ {deep} }}");
    }
    for body in [deep, "if index > initial { break; }".repeat(33)] {
        let module = parse_nuis_module(&source(&body)).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}
