use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn walk(input: bool, limit: i64) -> bool {{
            let flag = input;
            let saved = flag;
            let other = false;
            let __nuis_bool_seed_0 = false;
            let index: i64 = 0;
            while index < limit {{ let index: i64 = index + 1; {body} }}
            return flag;
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

#[test]
fn bool_carries_use_explicit_private_words_without_capture_name_collisions() {
    for body in [
        "let flag = flag;",
        "let flag = flag == false;",
        "let flag = saved;",
        "let flag = flag; let before = flag; if flag { let flag = false; } else { let flag = true; } let other = flag; if other { let flag = before; }",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let layouts = control_values::layouts(&module);
            assert!(scalar_helpers::collect_with_layouts(&module, &layouts)["walk"].may_loop);
            assert!(!scalar_helpers::collect(&module).contains_key("walk"));
            outline_buffer_loops(&mut module).unwrap();
            let iteration = module.functions.iter().find(|f| f.name.starts_with("__nuis_scalar_iteration_")).unwrap();
            assert!(!iteration.params.iter().any(|p| p.name == "flag" || p.name == "other" || p.name == "__nuis_bool_seed_0"));
            let words = iteration.params.iter().filter(|p| p.name.starts_with("__nuis_bool_seed_")).collect::<Vec<_>>();
            assert_eq!(words.len(), if body.contains("let other") { 2 } else { 1 });
            for word in words {
                assert_eq!(word.ty, scalar_type("i64"));
                assert!(iteration.body.iter().any(|s| matches!(s, NirStmt::Let { ty: Some(ty), value: NirExpr::CastI64ToBool(value), .. }
                    if ty == &scalar_type("bool") && value.as_ref() == &NirExpr::Var(word.name.clone()))));
            }
            let layout = module.structs.iter().find(|s| Some(scalar_type(&s.name)) == iteration.return_type).unwrap();
            assert!(layout.fields.iter().all(|f| f.ty == scalar_type("i64")));
        }
    }
}

#[test]
fn bool_carries_do_not_launder_const_parameters_or_forward_sibling_reads() {
    let base = source("let flag = flag == false;");
    for text in [
        base.replace("let flag = input;", "const flag: bool = input;"),
        base.replace("let flag = flag == false;", "let input = input == false;"),
        source("let flag = other; let other = flag;"),
        source("let flag = flag; let other = 1;"),
        source("let flag = flag; let flag: i64 = index;"),
        source("if index < limit { let local = true; } let flag = local;"),
        source("let flag = flag; const other: bool = flag;"),
    ] {
        assert!(
            !parse_nuis_module(&text).is_ok_and(|m| scalar_helpers::collect_with_layouts(
                &m,
                &control_values::layouts(&m)
            )
            .contains_key("walk")),
            "{text}"
        );
    }
}
