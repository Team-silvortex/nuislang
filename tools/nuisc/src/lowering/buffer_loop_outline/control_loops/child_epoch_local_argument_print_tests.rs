use super::*;

fn source(kind: &str) -> String {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/child_epoch_local_argument_print.ns"
    ))
    .replace("f64", kind)
}

fn admitted(source: &str, reversed: bool) -> bool {
    let mut module = crate::frontend::parse_nuis_module(source).unwrap();
    if reversed {
        module.functions.reverse();
    }
    let layouts = control_values::layouts(&module);
    scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("work")
}

#[test]
fn child_epoch_local_source_admits_only_fresh_ready_entries() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        for reversed in [false, true] {
            assert!(admitted(&source, reversed));
            assert!(admitted(
                &source.replace("epoch_word: i64", "epoch_word"),
                reversed
            ));
        }
    }
}

#[test]
fn child_epoch_local_source_rejects_branch_and_ancestor_grants() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        let init = "let mut current = Pair { total: total, code: left, flag: selected };";
        let stale = source
            .replace(init, "")
            .replace("while outer < 1 {", &format!("{init} while outer < 1 {{"));
        let inherited = stale.replace(
            "total = finish(total, 1);",
            "total = finish(total, 1); print(observe(current.total));",
        );
        let branch = source
            .replace(
                "let epoch_word: i64 =",
                "if selected { let epoch_word: i64 =",
            )
            .replace("print(74);", "} print(74);");
        let grandchild = source
            .replace(
                "print(71);",
                "let mut nested = 0; while nested < 1 { print(71);",
            )
            .replace("print(75);", "print(75); nested = nested + 1; }");
        for bad in [stale, inherited, branch, grandchild] {
            assert_ne!(bad, source);
            for reversed in [false, true] {
                assert!(!admitted(&bad, reversed));
            }
        }
    }
}

#[test]
fn child_epoch_local_source_keeps_aliases_duplicates_and_hidden_roots_closed() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        let helper =
            "@noinline fn bool_word(value:bool)->i64{if value{return 1;}return 0;} fn main()";
        for bad in [
            source.replace("print(74);", "let alias = epoch_word;"),
            source.replace("print(74);", "print(epoch_word);"),
            source
                .replace("let epoch_word:", "let mut epoch_word:")
                .replace("print(74);", "epoch_word = 1;"),
            source.replace(
                "checked64(epoch_word, lazy_guard)",
                "checked64(epoch_word ^ epoch_word, lazy_guard)",
            ),
            source.replace(
                "current.flag && (checked64(epoch_word, lazy_guard) != 0)",
                "checked64(epoch_word, lazy_guard) != 0",
            ),
            source.replace("fn main()", helper).replace(
                "(i64_from_i32(current.code) + 4294967296) ^ mask",
                "i64_from_i32(current.code) ^ bool_word((initial_guard > 0) && (lazy_guard > 0))",
            ),
        ] {
            assert_ne!(bad, source);
            for reversed in [false, true] {
                assert!(!admitted(&bad, reversed));
            }
        }
    }
}
