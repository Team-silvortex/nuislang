use super::*;
use crate::frontend::parse_nuis_module;

const ARM: &str = "if index > initial {
    if index < bound { let total: i64 = total + index; }
    else { let total: i64 = total * index; }
  } else {
    if index == initial { let total: i64 = total - index; }
  }";

fn source(arm: &str) -> String {
    carries_tests::SOURCE.replace("let total: i64 = total + index;", arm)
}

#[test]
fn nested_decisions_with_distinct_leaves_use_scoped_helpers_not_two_value_collapse() {
    for arm in [
        ARM,
        "if index > initial { if index < bound { let total: i64 = total + index; } }",
        "if index > initial {} else { if index < bound {} else { let total: i64 = total * index; } }",
        "if index > initial { let total: i64 = total + index; } else if index < bound { let total: i64 = total * index; } else { let total: i64 = total - index; }",
        "if index > initial && index < bound { if initial <= index || bound == index { let total: i64 = total + index; } }",
    ] {
        for reverse in [false, true] {
            let mut module = parse_nuis_module(&source(arm)).unwrap();
            if reverse { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            for name in ["walk", "wrap", "choose"] { assert!(catalog[name].may_loop, "{arm}"); }
            assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
            let outlined = outline_buffer_loops(&mut module).unwrap();
            let iteration = module.functions.iter().find(|f| f.name.starts_with("__nuis_scalar_iteration_")).unwrap();
            assert!(outlined.functions.contains(&iteration.name));
            assert!(outlined.guarded_functions.len() > 2);
            assert!(matches!(&iteration.body[0], NirStmt::Let { name, .. } if name == "index"));
            let walk = module.functions.iter().find(|f| f.name == "walk").unwrap();
            let body = walk.body.iter().find_map(|s| match s { NirStmt::While { body, .. } => Some(body), _ => None }).unwrap();
            assert!(matches!(&body[0], NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == &iteration.name));
            assert!(matches!(body.last(), Some(NirStmt::Let { name, .. }) if name == "index"));
            assert!(!body[..body.len()-1].iter().any(|s| matches!(s, NirStmt::Let { name, .. } if name == "index")));
        }
    }
}

#[test]
fn nested_arms_check_every_path_for_order_type_effect_and_header_safety() {
    for (from, to) in [
        ("index < bound", "total < bound"),
        ("index < bound", "checksum < bound"),
        ("index < bound", "index < total"),
        ("index < bound", "index < (checksum / initial)"),
        ("index == initial", "index == wrap(initial, bound, step)"),
        ("total - index", "total + checksum"),
        ("total - index", "total / checksum"),
        ("total - index", "total % checksum"),
        (
            "let total: i64 = total - index;",
            "let total: i64 = total - index; print(total);",
        ),
        ("let total: i64 = total - index;", "break;"),
        ("let total: i64 = total - index;", "continue;"),
        (
            "let total: i64 = total - index;",
            "while index < bound { let index: i64 = index + 1; }",
        ),
    ] {
        let module = parse_nuis_module(&source(&ARM.replace(from, to))).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{from} => {to}");
        assert!(!catalog.contains_key("choose"), "{from} => {to}");
    }
    for (from, to) in [
        ("let total: i64 = initial;", "const total: i64 = initial;"),
        ("total", "limit"),
        ("total", "stride"),
    ] {
        let module = parse_nuis_module(&source(ARM).replace(from, to)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("choose"), "{from} => {to}");
    }
    assert!(parse_nuis_module(&source(&ARM.replace("total - index", "false"))).is_err());
    assert!(parse_nuis_module(&source(&ARM.replace(
        "let total: i64 = total - index",
        "let total: i32 = total - index"
    )))
    .is_err());
}

#[test]
fn nested_normalization_has_a_depth_guard_without_weakening_native_limits() {
    for depth in [2, 8, 32, 33] {
        let mut arm = "let total: i64 = total + index;".to_owned();
        for _ in 0..depth {
            arm = format!("if index > initial {{ {arm} }}");
        }
        let module = parse_nuis_module(&source(&arm)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert_eq!(catalog.contains_key("walk"), depth <= 32, "depth {depth}");
    }
}
