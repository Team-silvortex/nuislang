use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    carries_tests::SOURCE.replace(
        "let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
        body,
    )
}

const BODY: &str = "
    let total: i64 = total + index;
    if total < bound {
        let total: i64 = total * 2;
        let checksum: i64 = checksum + total;
        if checksum < bound && index > initial {
            let total: i64 = total - 1;
            let checksum: i64 = checksum + total;
        }
    } else { let checksum: i64 = checksum - total; }
    let total: i64 = total + checksum;
    let checksum: i64 = checksum + total;
";

#[test]
fn statement_sequences_outline_once_and_return_unique_carries() {
    for body in [
        BODY,
        "if index > initial { let total: i64 = total + index; let total: i64 = total * index; }",
        "if index > initial { let total: i64 = total + index; } else { let checksum: i64 = checksum + index; }",
        "let total: i64 = total + index; let total: i64 = total * 2;",
        "if index < bound { let total: i64 = total + index; let checksum: i64 = checksum + total; }",
        "if index < bound { let total: i64 = total + index; } else { let checksum: i64 = checksum + index; } let checksum: i64 = checksum + total;",
        // The old duplicate-write rejection is now an accepted source-order case.
        "let total: i64 = total + index; let total: i64 = total + index;",
        "if index > initial { let total: i64 = total + index; } else { let total: i64 = total + index; let total: i64 = total * index; } let checksum: i64 = checksum * total;",
    ] {
        for reverse in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reverse { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            for name in ["walk", "wrap", "choose"] {
                assert!(catalog.get(name).is_some_and(|f| f.may_loop), "{name}: {body}");
            }
            assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
            let result = outline_buffer_loops(&mut module).unwrap();
            let iterations = module.functions.iter().filter(|f| f.name.starts_with("__nuis_scalar_iteration_")).collect::<Vec<_>>();
            assert_eq!(iterations.len(), 1, "{body}");
            let iteration = iterations[0];
            assert!(result.functions.contains(&iteration.name));
            assert!(matches!(&iteration.body[0], NirStmt::Let { name, .. } if name == "index"));
            let returned = iteration.body.last().unwrap();
            if body.contains("let checksum") {
                let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) = returned else { panic!("{returned:?}"); };
                assert_eq!(fields.len(), 2, "one slot per carry, not per write");
            } else {
                assert!(matches!(returned, NirStmt::Return(Some(NirExpr::Var(name))) if name == "total"));
            }
        }
    }
}

#[test]
fn statement_sequences_check_unselected_arms_and_keep_header_immutable() {
    for invalid in [
        "let total: i64 = total + checksum;", // Still a forward sibling read.
        "let total: i64 = total / index;",
        "let total: i64 = total % index;",
        "let total: i64 = wrap(index, limit, stride);",
        "let index: i64 = index + 1;",
        "let limit: i64 = limit + 1;",
        "let stride: i64 = stride + 1;",
        "let bound: i64 = bound + 1;", // A parameter, not a mutable local.
        "let fresh: i64 = index;",     // New iteration-local names are a later slice.
        "print(total);",
        "break;",
        "continue;",
        "while index < limit { let index: i64 = index + 1; }",
    ] {
        let body = format!("if index < initial {{ {invalid} let checksum: i64 = checksum + index; }} else {{ let total: i64 = total + index; }}");
        let module = parse_nuis_module(&source(&body)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{invalid}");
        assert!(!catalog.contains_key("choose"), "{invalid}");
    }
    for from_to in [
        ("let total: i64 = initial;", "const total: i64 = initial;"),
        ("total < bound", "total < checksum"),
        ("checksum < bound", "checksum < (bound / initial)"),
    ] {
        let module = parse_nuis_module(&source(BODY).replace(from_to.0, from_to.1)).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("choose")
        );
    }
}

#[test]
fn statement_sequence_availability_is_branch_local_until_join() {
    // Updating a sibling in the THEN arm cannot make it available in ELSE.
    let bad = "if index < bound { let checksum: i64 = checksum + index; let total: i64 = total + checksum; } else { let total: i64 = total + checksum; }";
    let module = parse_nuis_module(&source(bad)).unwrap();
    assert!(
        !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
            .contains_key("walk")
    );
    let good = "if index < bound { let checksum: i64 = checksum + index; } else { let total: i64 = total + index; } let total: i64 = total + checksum;";
    let module = parse_nuis_module(&source(good)).unwrap();
    assert!(
        scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
            .contains_key("walk")
    );
}
