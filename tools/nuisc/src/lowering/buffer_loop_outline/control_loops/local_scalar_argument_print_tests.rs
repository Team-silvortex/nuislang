use super::*;

fn source(kind: &str) -> String {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/local_scalar_argument_print.ns"
    ))
    .replace("f64", kind)
}

fn admitted(source: &str, reversed: bool) -> bool {
    let Ok(mut module) = crate::frontend::parse_nuis_module(source) else {
        return false;
    };
    if reversed {
        module.functions.reverse();
    }
    let layouts = control_values::layouts(&module);
    scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("work")
}

fn replace_once(source: &str, old: &str, new: &str) -> String {
    assert_eq!(source.matches(old).count(), 1, "fixture anchor: {old}");
    source.replacen(old, new, 1)
}

#[test]
fn local_scalar_argument_print_accepts_ready_initializers_and_branch_local_uses() {
    for kind in ["f32", "f64"] {
        for reversed in [false, true] {
            let source = source(kind);
            assert!(admitted(&source, reversed));
            let typed = source
                .replace("let parameter_word =", "let parameter_word: i64 =")
                .replace("let ready_word =", "let ready_word: i64 =")
                .replace("let branch_word =", "let branch_word: i64 =");
            assert!(admitted(&typed, reversed));
        }
    }
}

#[test]
fn local_scalar_argument_print_does_not_lend_epoch_reads_to_initializers() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        for value in [
            "i64_from_i32(current.code)",
            "i64_from_i32(current.code) ^ parameter_word",
            "checked64(i64_from_i32(current.code) ^ parameter_word, lazy_guard)",
        ] {
            // The preceding print has certified this field, not ordinary readiness.
            let bad = replace_once(
                &source,
                "current = Pair { total: -current.total, code: right, flag: selected };",
                &format!("let premature = {value}; current = Pair {{ total: -current.total, code: right, flag: selected }};"),
            );
            for reversed in [false, true] {
                assert!(!admitted(&bad, reversed), "{kind}: {value}");
            }
        }
        let unready = replace_once(
            &source,
            "let parameter_word = checked64((i64_from_i32(left) + 4294967296) ^ mask, initial_guard);",
            "let parameter_word = checked64((i64_from_i32(current.code) + 4294967296) ^ mask, initial_guard);",
        );
        assert!(!admitted(&unready, false));
    }
}

#[test]
fn local_scalar_argument_print_rejects_forward_reads_and_branch_escape() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        for bad in [
            replace_once(&source, "print(71);", "print(ready_word);"),
            replace_once(
                &source,
                "index = index + 1;",
                "print(branch_word); index = index + 1;",
            ),
            replace_once(&source, "let ready_word =", "let ready_word: i32 ="),
        ] {
            for reversed in [false, true] {
                assert!(!admitted(&bad, reversed));
            }
        }
        for (marker, name) in [
            ("print(71);", "ready_word"),
            ("index = index + 1;", "branch_word"),
        ] {
            let bad = replace_once(&source, marker, &format!("print({name}); {marker}"));
            assert!(crate::frontend::parse_nuis_module(&bad)
                .unwrap_err()
                .contains(&format!("unknown value `{name}`")));
        }
    }
}

#[test]
fn local_scalar_argument_print_keeps_child_epoch_reads_print_only() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        let child = "let mut child = 0; while child < 1 {
            print(observe(choose(current.total, current.flag && (i64_from_i32(current.code) != 0))));
            current = Pair { total: -current.total, code: right, flag: selected };
            child = child + 1;
        }";
        let ready_child = replace_once(&source, "print(ready_word);", child);
        assert!(admitted(&ready_child, false));
        let premature = replace_once(
            &ready_child,
            "current = Pair { total: -current.total, code: right, flag: selected };\n            child",
            "let premature = i64_from_i32(current.code) ^ ready_word;
            current = Pair { total: -current.total, code: right, flag: selected };
            child",
        );
        assert!(!admitted(&premature, false));
    }
}
