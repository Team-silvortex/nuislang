use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    carries_tests::SOURCE.replace(
        "let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
        body,
    )
}

#[test]
fn literal_loops_use_existing_scopes_and_keep_child_declarations_local() {
    for body in [
        "while total < limit { let total: i64 = total + 1; } let checksum = checksum + total;",
        "let child: i64 = 0; while child < index { let child: i64 = child + 1; let total = total + child; } let checksum = checksum + total;",
        "let child = 0; let flag = true; while child < index { let child = child + 1; if flag { let total = total + child; } }",
        "let child: i64 = 0; while child < index { let child: i64 = child + 1; let local = child; let total = total + local; }",
        "let child: i64 = 0; if index < bound { while child < index { let child = child + 1; let total = total + child; } } let checksum = checksum + child;",
        "let child: i64 = index; while child > 0 { let child = child - stride; let total = total; let flag = true; let flag = false; let pair = Pair { seed: child, value: child }; let pair = Pair { seed: pair.seed, value: total }; let total = total + pair.value; }",
        "let child: i64 = 0; while child < index { let child = child + 1; let grandchild: i64 = 0; while grandchild < child { let grandchild = grandchild + 1; let total = total + grandchild; } }",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            for name in ["walk", "wrap", "choose"] {
                assert!(catalog.get(name).is_some_and(|f| f.may_loop), "{name}: {body}");
            }
            assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
            outline_buffer_loops(&mut module).unwrap();
            let outer = module.functions.iter().find(|f| f.name == "__nuis_scalar_iteration_0").unwrap();
            for name in ["child", "grandchild", "local", "flag", "pair"] {
                assert!(!outer.params.iter().any(|p| p.name == name), "child-local capture: {name}");
            }
        }
    }
}

#[test]
fn literal_loops_cannot_launder_headers_write_authority_or_future_siblings() {
    for effects in [
        "let index = index + 1;",
        "if false { let index = index + 1; }",
        "let limit = limit + 1;",
        "let stride = stride + 1;",
        "let bound = bound + 1;",
        "let child = child + 1;",
        "let inner_limit = inner_limit + 1;",
        "let inner_step = inner_step + 1;",
        "let total = total + checksum; let checksum = checksum + child;",
        "let saved = checksum; let total = total + saved;",
        "let total = true;",
        "print(child);",
    ] {
        let text = source(&format!(
            "let child: i64 = 0; let inner_limit = index; let inner_step: i64 = 1;
            while child < inner_limit {{ let child = child + inner_step; {effects} }}
            let checksum = checksum + 1;"
        ));
        let module = match parse_nuis_module(&text) {
            Ok(module) => module,
            Err(error) => {
                assert!(error.contains("type"), "{effects}: {error}");
                continue;
            }
        };
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{effects}");
        assert!(!catalog.contains_key("choose"), "{effects}");
    }
    for body in [
        "while bound < limit { let bound = bound + 1; }",
        "let child = 0; while child < checksum { let child = child + 1; } let checksum = checksum + 1;",
        "let child = 0; while child < index / stride { let child = child + 1; }",
        "let child = 0; while child < index { let child = child + stride / 1; }",
        "let child = 0; while child < index { let child = child + child; }",
    ] {
        let module = parse_nuis_module(&source(body)).unwrap();
        assert!(!scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module)).contains_key("walk"), "{body}");
    }
}

#[test]
fn literal_loop_scope_and_combined_branch_loop_depth_are_bounded() {
    for suffix in [
        "let total = total + local;",
        "if index > 0 { let total = total + local; }",
    ] {
        let error = parse_nuis_module(&source(&format!(
            "let child = 0; while child < index {{ let child = child + 1; let local = child; }} {suffix}"
        ))).unwrap_err();
        assert!(error.contains("unknown value `local`"), "{error}");
    }
    for depth in [1, 16, 32, 33] {
        let mut body = "let total = total + index;".to_owned();
        for level in 0..depth {
            body = if level % 2 == 0 {
                format!("let child{level} = 0; while child{level} < index {{ let child{level} = child{level} + 1; {body} }}")
            } else {
                format!("if index > 0 {{ {body} }}")
            };
        }
        let module = parse_nuis_module(&source(&body)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert_eq!(catalog.contains_key("walk"), depth <= 32, "depth {depth}");
    }
}
