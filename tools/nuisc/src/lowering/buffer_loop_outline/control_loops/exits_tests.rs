use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    carries_tests::SOURCE.replace(
        "let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
        body,
    )
}

#[test]
fn leading_step_exits_reuse_scoped_guards_and_keep_child_control_local() {
    for (body, breaks) in [
        ("break;", 1),
        ("continue;", 0),
        ("if index > initial { break; } let total = total + index;", 1),
        ("if index > initial { continue; } let total = total + index;", 0),
        ("if index > initial { let total = total + index; break; } else { let total = total - index; continue; }", 1),
        ("let total = total + index; if index > initial { if index < bound { break; } } if index == bound { continue; } let checksum = checksum + total;", 1),
        ("let child = 0; while child < index { let child = child + 1; if child > 1 { break; } let total = total + child; } let checksum = checksum + child;", 1),
        ("let child = 0; while child < index { let child = child + 1; if child > 1 { continue; } let total = total + child; } if index > initial { break; }", 1),
        ("let child = 0; while child < index { let child = child + 1; if child > 1 { break; } } if index > initial { break; }", 2),
    ] {
        let mut module = parse_nuis_module(&source(body)).unwrap();
        let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        for name in ["walk", "wrap", "choose"] {
            assert!(catalog.get(name).is_some_and(|f| f.may_loop), "{name}: {body}");
        }
        assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert_eq!(outlined.break_controls.len(), breaks, "{body}");
        for (name, flag) in &outlined.break_controls {
            let helper = module.functions.iter().find(|f| &f.name == name).unwrap();
            assert!(helper.params.iter().any(|p| &p.name == flag && p.ty == scalar_type("i64")));
            let Some(NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) = helper.body.last() else { panic!("break must return canonical state"); };
            assert_eq!(fields.last().unwrap().1, NirExpr::Var(flag.clone()));
        }
    }
}

#[test]
fn leading_step_exit_recovery_avoids_future_source_bindings() {
    let text = source("if index > initial { break; } let total = total + index;").replace(
        "return index + total + checksum;",
        "let __nuis_advanced_index_0 = true; return index + total + checksum;",
    );
    let mut module = parse_nuis_module(&text).unwrap();
    outline_buffer_loops(&mut module).unwrap();
    let walk = module.functions.iter().find(|f| f.name == "walk").unwrap();
    let (name, _) = walk
        .body
        .iter()
        .find_map(|s| match s {
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Var(value),
            } if name.starts_with("__nuis_advanced_index_") && value == "index" => Some((name, ty)),
            _ => None,
        })
        .expect("entry seed");
    assert_ne!(name, "__nuis_advanced_index_0");
    assert!(walk.body.iter().any(|s| matches!(s, NirStmt::Let { name: target, value: NirExpr::Var(value), .. } if target == "index" && value == name)));
}

#[test]
fn leading_step_exits_do_not_admit_extra_steps_or_hidden_invalid_effects() {
    for body in [
        "let index = index + stride; continue;",
        "if index > initial { let index = index + stride; continue; }",
        "if false { let limit = limit + 1; break; }",
        "if false { let stride = stride + 1; continue; }",
        "if index > initial { let total = total + checksum; break; } let checksum = checksum + 1;",
        "if index > initial { print(index); break; }",
        "if index > initial { break; let total = total + index; }",
        "if index > initial { continue; let total = total + index; }",
        "let child = 0; while child < index { let child = child + 1; if child > 1 { let index = index + 1; break; } }",
    ] {
        let module = parse_nuis_module(&source(body)).unwrap();
        assert!(!scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module)).contains_key("walk"), "{body}");
    }
    let mut deep = "break;".to_owned();
    for _ in 0..33 {
        deep = format!("if index > initial {{ {deep} }}");
    }
    let module = parse_nuis_module(&source(&deep)).unwrap();
    assert!(
        !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
            .contains_key("walk")
    );
    let sequential = "if index > initial { break; }".repeat(33);
    let module = parse_nuis_module(&source(&sequential)).unwrap();
    assert!(
        !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
            .contains_key("walk")
    );
}
