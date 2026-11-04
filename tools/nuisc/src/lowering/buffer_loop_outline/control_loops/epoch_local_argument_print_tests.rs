use super::*;

fn source(kind: &str) -> String {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/native_application_bridge/epoch_local_argument_print.ns"
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

#[test]
fn epoch_local_argument_print_source_admits_root_pairs_without_erasing_definitions() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        for reversed in [false, true] {
            assert!(admitted(&source, reversed));
            let early =
                source.replace("print(75);", "print(75); if selected { break; } print(79);");
            assert!(admitted(&early, reversed));
            assert!(!admitted(
                &early.replace("print(71);", "let __nuis_buffer_break: i64 = 0; print(71);"),
                reversed
            ));
            assert!(admitted(
                &source.replace("epoch_word: i64", "epoch_word"),
                reversed
            ));
        }
        let module = crate::frontend::parse_nuis_module(&source).unwrap();
        let work = module.functions.iter().find(|f| f.name == "work").unwrap();
        let body = work
            .body
            .iter()
            .find_map(|stmt| match stmt {
                NirStmt::While { body, .. } => Some(body),
                _ => None,
            })
            .unwrap();
        assert!(
            matches!(&body[1], NirStmt::Let { name, value:NirExpr::Call { callee, .. }, .. }
            if name == "epoch_word" && callee == "checked64")
        );
        assert!(matches!(&body[2], NirStmt::Print(_)));
    }
}

#[test]
fn epoch_local_argument_print_source_keeps_unproved_uses_and_scopes_closed() {
    for kind in ["f32", "f64"] {
        let source = source(kind);
        for bad in [
            source.replace("print(74);", "print(epoch_word);"),
            source.replace("print(74);", "let alias = epoch_word;"),
            source.replace("print(74);", "epoch_word = 1;"),
            source.replace("epoch_word: i64", "epoch_word: i32"),
            source.replace(
                "current.flag && (checked64(epoch_word, lazy_guard) != 0)",
                "checked64(epoch_word, lazy_guard) != 0",
            ),
            source.replace(
                "checked64(epoch_word, lazy_guard)",
                "checked64(epoch_word ^ epoch_word, lazy_guard)",
            ),
            source.replace("print(71);", "let before = 1;"),
            source
                .replace(
                    "let epoch_word: i64 =",
                    "if selected { let epoch_word: i64 =",
                )
                .replace("print(74);", "} print(74);"),
        ] {
            assert_ne!(bad, source);
            for reversed in [false, true] {
                assert!(!admitted(&bad, reversed));
            }
        }
        let helper =
            "@noinline fn bool_word(value:bool)->i64{if value{return 1;}return 0;} fn main()";
        let logical_initializer = source.replace("fn main()", helper).replace(
            "(i64_from_i32(current.code) + 4294967296) ^ mask",
            "i64_from_i32(current.code) ^ bool_word((initial_guard > 0) && (lazy_guard > 0))",
        );
        assert_ne!(logical_initializer, source);
        crate::frontend::parse_nuis_module(&logical_initializer).unwrap();
        for reversed in [false, true] {
            assert!(!admitted(&logical_initializer, reversed));
        }
    }
}
