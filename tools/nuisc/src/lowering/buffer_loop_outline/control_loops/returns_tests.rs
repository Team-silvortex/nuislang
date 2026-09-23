use super::*;
use crate::frontend::parse_nuis_module;

fn source(result: &str, value: &str, fallback: &str, effects: &str) -> String {
    format!(
        "mod cpu Main {{
        struct Pair {{ value: i64, index: i64 }}
        struct Other {{ value: i64, index: i64 }}
        fn work(limit: i64, stride: i64, stop: i64) -> {result} {{
            let index = 0;
            while index < limit {{
                {effects}
                if index == stop {{ return {value}; }}
                let index = index + stride;
            }}
            return {fallback};
        }}
        fn wrap(limit: i64, stride: i64, stop: i64) -> {result} {{
            return work(limit, stride, stop);
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

#[test]
fn counted_returns_admit_exact_values_without_widening_buffer_helpers() {
    for (result, value, fallback) in [
        ("i64", "index / stride", "-1"),
        ("bool", "index == stop", "false"),
        (
            "Pair",
            "Pair { value: index / stride, index: index }",
            "Pair { value: -1, index: -1 }",
        ),
    ] {
        for leading in [false, true] {
            let mut text = source(result, value, fallback, "");
            if leading {
                text = text.replace("let index = index + stride;", "").replace(
                    "while index < limit {",
                    "while index < limit { let index = index + stride;",
                );
            }
            let mut module = parse_nuis_module(&text).unwrap();
            let layouts = control_values::layouts(&module);
            for _ in 0..2 {
                let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
                assert!(catalog.get("work").is_some_and(|f| f.may_loop), "{text}");
                assert!(catalog["wrap"].may_loop);
                module.functions.reverse();
            }
            assert!(!scalar_helpers::collect(&module).contains_key("work"));
            let outlined = outline_buffer_loops(&mut module).unwrap();
            assert_eq!(outlined.break_controls.len(), 1);
            assert!(outlined.functions.contains("work"));
            assert!(!outlined.guarded_functions.is_empty());
        }
    }
}

#[test]
fn counted_returns_propagate_child_exit_and_reserve_future_source_names() {
    let text = source("i64", "index", "-1", "")
        .replace("if index == stop { return index; }", "let child = 0; while child < stop { if child == 1 { return index + child; } let child = child + 1; }")
        .replace("return -1;", "let __nuis_return_pending_0 = 9; let __nuis_return_value_0 = 11; return __nuis_return_pending_0 + __nuis_return_value_0;");
    let mut module = parse_nuis_module(&text).unwrap();
    let layouts = control_values::layouts(&module);
    assert!(scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("work"));
    let f = module.functions.iter().find(|f| f.name == "work").unwrap();
    let body = returns::normalize(f, &layouts).unwrap().unwrap();
    let NirStmt::Let { name, .. } = &body[0] else {
        panic!("seed")
    };
    assert_ne!(name, "__nuis_return_pending_0");
    let NirStmt::Let { name, .. } = &body[1] else {
        panic!("payload")
    };
    assert_ne!(name, "__nuis_return_value_0");
    assert_eq!(
        outline_buffer_loops(&mut module)
            .unwrap()
            .break_controls
            .len(),
        2
    );
}

#[test]
fn counted_returns_reject_wrong_types_effects_writes_and_unreachable_suffixes() {
    let base = source("i64", "index", "-1", "");
    for text in [
        base.replace("return index;", "return true;"),
        base.replace("return index;", "return;"),
        base.replace("return index;", "print(index); return index;"),
        base.replace("return index;", "return index; let extra = 1;"),
        base.replace("return index;", "let limit = 0; return index;"),
        base.replace("let index = 0;", "const index: i64 = 0;"),
        base.replace("return index;", "let index = index + stride; return index;"),
        base.replace("index + stride", "index + index"),
        source(
            "Pair",
            "Other { value: index, index: index }",
            "Pair { value: 0, index: 0 }",
            "",
        ),
        base.replace(
            "while index < limit {",
            "let sibling = 0; while index < limit {",
        )
        .replace("return index;", "return sibling;")
        .replace(
            "let index = index + stride;",
            "let sibling = sibling + 1; let index = index + stride;",
        ),
    ] {
        let module = parse_nuis_module(&text).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("work"),
            "{text}"
        );
    }
    let nested = format!(
        "{}return index;{}",
        "if index == stop {".repeat(33),
        "}".repeat(33)
    );
    for text in [
        base.replace("return index;", &nested),
        base.replace(
            "if index == stop { return index; }",
            &"if index == stop { return index; }".repeat(33),
        ),
    ] {
        let module = parse_nuis_module(&text).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("work")
        );
    }
}

#[test]
fn counted_returns_do_not_bind_unknown_source_or_nir_references() {
    let base = source("i64", "index", "-1", "");
    for payload in [false, true] {
        let (name, invalid) = if payload {
            (
                "__nuis_return_value_0",
                base.replace("return index;", "return __nuis_return_value_0;"),
            )
        } else {
            (
                "__nuis_return_pending_0",
                base.replace("let index = 0;", "let index = __nuis_return_pending_0;"),
            )
        };
        assert!(parse_nuis_module(&invalid)
            .unwrap_err()
            .contains("unknown value"));
        let mut module = parse_nuis_module(&base).unwrap();
        let function = module
            .functions
            .iter_mut()
            .find(|f| f.name == "work")
            .unwrap();
        if payload {
            let NirStmt::While { body, .. } = &mut function.body[1] else {
                panic!("loop")
            };
            let NirStmt::If { then_body, .. } = &mut body[0] else {
                panic!("guard")
            };
            then_body[0] = NirStmt::Return(Some(NirExpr::Var(name.to_owned())));
        } else {
            let NirStmt::Let { value, .. } = &mut function.body[0] else {
                panic!("seed")
            };
            *value = NirExpr::Var(name.to_owned());
        }
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("work")
        );
    }
}
