use super::*;
use crate::frontend::parse_nuis_module;

#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/return_typed_exit_prefix_cases.rs"]
mod cases;
use cases::SOURCE;

fn rewrite(source: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

#[test]
fn return_typed_exit_prefix_preserves_six_data_fields_one_tail_and_input_bound() {
    for source in [SOURCE.to_owned(), cases::mirror_source()] {
        let (module, names) = rewrite(&source);
        assert!(!names.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let records = module
            .structs
            .iter()
            .filter(|s| s.name.starts_with("__nuis_effect_fields_value"))
            .collect::<Vec<_>>();
        assert!(!records.is_empty());
        for record in records {
            assert_eq!(
                record
                    .fields
                    .iter()
                    .map(|f| f.ty.name.as_str())
                    .collect::<Vec<_>>(),
                ["i64", "i32", "f32", "f64", "bool", "bool"]
            );
        }
        assert!(module
            .functions
            .iter()
            .filter(|f| names.contains(&f.name))
            .all(|f| f.params.len() <= 32));
        let count = |callee: &str| {
            module
                .functions
                .iter()
                .flat_map(|f| &f.body)
                .filter(|s| {
                    matches!(s, NirStmt::Let {value: NirExpr::Call {callee: c, ..}, ..}
                | NirStmt::Return(Some(NirExpr::Call {callee: c, ..})) if c == callee)
                })
                .count()
        };
        assert_eq!(count("select"), 1);
        assert_eq!(count("tail_work"), 1);
        assert_eq!(count("exit_owner"), 1);
        assert_eq!(count("sibling_owner"), 1);
        for callee in [
            "exit_small",
            "exit_narrow",
            "exit_wide",
            "exit_flag",
            "exit_peer",
        ] {
            assert_eq!(count(callee), 2);
        }
    }
}

#[test]
fn return_typed_exit_prefix_has_unmasked_current_return_operand_witnesses() {
    for field in 0..6 {
        assert!(
            cases::witnesses().iter().any(|case| {
                let current = case.fields();
                let mut stale = current;
                stale[field] = case.before()[field];
                cases::answer(current) != cases::answer(stale)
            }),
            "no unmasked current operand witness at field {field}"
        );
    }
    for pick in [false, true] {
        let exits = cases::cli_cases()
            .into_iter()
            .filter(|c| c.pick == pick && c.take)
            .collect::<Vec<_>>();
        assert!(exits.iter().any(|c| c.result() == 0));
        assert!(exits.iter().any(|c| c.result() == 1));
    }
    assert_eq!(cases::cases().len(), 160);
    assert_eq!(cases::cli_cases().len(), 16);
    assert_eq!(cases::failures().len(), 12);
    for (case, slot, stage) in cases::failures() {
        assert_eq!(case.words()[slot], 1, "{stage}");
    }
}

#[test]
fn return_typed_exit_prefix_keeps_atomic_authority_and_shape_vetoes() {
    for source in [
        SOURCE.replacen(
            "let local = exit_owner(local, 1 / yes_owner);",
            "let seed = exit_owner(local, 1 / yes_owner);",
            1,
        ),
        SOURCE.replacen(
            "if inner_condition(inner, 1 / choice)",
            "if inner { return d; } if inner_condition(inner, 1 / choice)",
            1,
        ),
        SOURCE.replacen(
            "let a = exit_small(a, 1 / small_proof);",
            "const a: i32 = exit_small(a, 1 / small_proof);",
            1,
        ),
        SOURCE.replacen(
            "let b = exit_narrow(b, a, 1 / narrow_proof);",
            "let b: f64 = c;",
            1,
        ),
        SOURCE.replace("let d = flag_seed;", "const d: bool = flag_seed;"),
        SOURCE.replacen(
            "let local = exit_owner(local, 1 / yes_owner);",
            "let a = exit_small(a, 1 / small_proof);",
            1,
        ),
    ] {
        let mut module = match parse_nuis_module(&source) {
            Ok(module) => module,
            Err(error) => {
                assert!(error.contains("expects `f32`, found `f64`"), "{error}");
                continue;
            }
        };
        let original = module.clone();
        let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
        let original_names = names.clone();
        assert!(outline(&mut module, &mut names).is_empty());
        assert_eq!(module, original);
        assert_eq!(names, original_names);
    }
}

#[test]
fn return_typed_exit_prefix_charges_original_work_and_inherits_depth() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let signatures = module
        .functions
        .iter()
        .filter_map(|f| {
            Some((
                f.name.clone(),
                (
                    f.params.iter().map(|p| p.ty.clone()).collect(),
                    f.return_type.clone()?,
                ),
            ))
        })
        .collect();
    let NirStmt::If { then_body, .. } = &function.body[0] else {
        panic!()
    };
    let budget = |nodes, expressions, logical_edges| nested::Budget {
        nodes,
        expressions,
        logical_edges,
    };
    let mut full = budget(64, 256, 32);
    let result = prove(then_body, &scope, &signatures, &mut full, 1).unwrap();
    let nodes = 64 - full.nodes;
    let expressions = 256 - full.expressions;
    assert_eq!(nodes, 39);
    assert_eq!(32 - full.logical_edges, 1);
    assert!(["local", "a", "b", "c", "d", "e", "finished"]
        .iter()
        .all(|n| !result.inputs.contains(*n)));
    for mut limited in [
        budget(nodes - 1, 256, 32),
        budget(64, expressions - 1, 32),
        budget(64, 256, 0),
    ] {
        assert!(prove(then_body, &scope, &signatures, &mut limited, 1).is_none());
    }
    assert!(prove(
        then_body,
        &scope,
        &signatures,
        &mut budget(nodes, expressions, 1),
        4
    )
    .is_some());
    assert!(prove(then_body, &scope, &signatures, &mut budget(64, 256, 32), 5).is_none());
}
