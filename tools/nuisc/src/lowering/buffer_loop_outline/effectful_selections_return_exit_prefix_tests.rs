use super::*;
use crate::frontend::parse_nuis_module;

const SOURCE: &str = include_str!("../../../tests/native_application_bridge/return_exit_prefix.ns");
const PREFIX: &str = "let local = exit_owner(local, 1 / work);";

fn rewrite(source: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

fn reject(source: &str) {
    let Ok(mut module) = parse_nuis_module(source) else {
        return;
    };
    let original = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let original_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty(), "{source}");
    assert_eq!(module, original);
    assert_eq!(names, original_names);
}

#[test]
fn return_exit_prefix_uses_original_certificate_and_one_common_tail() {
    for source in [
        SOURCE.to_owned(),
        SOURCE.replace("select(pick,", "select(pick != true,"),
    ] {
        let (module, generated) = rewrite(&source);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert!(module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
            .all(|f| f.params.len() <= 32));
        let calls = |callee: &str| {
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
        assert_eq!(calls("select"), 1);
        assert_eq!(calls("tail_work"), 1);
        assert_eq!(calls("exit_owner"), 2);
        assert_eq!(calls("exit_small"), 2);
        for record in module
            .structs
            .iter()
            .filter(|s| s.name.starts_with("__nuis_effect_fields_value"))
        {
            assert_eq!(
                record
                    .fields
                    .iter()
                    .map(|f| f.ty.name.as_str())
                    .collect::<Vec<_>>(),
                ["i64", "i32", "bool"]
            );
        }
    }
}

#[test]
fn return_exit_prefix_rejects_new_authority_types_seals_flat_children_and_exits() {
    for replacement in [
        "let seed = exit_owner(local, 1 / work);",
        "let a = exit_small(a, 1 / work);",
        "const local: i64 = exit_owner(local, 1 / work);",
        "if inner { let seed = local; }",
        "if inner { let a: f32 = 0.0; }",
        "if inner { return d; }",
        "return d;",
    ] {
        reject(&SOURCE.replacen(PREFIX, replacement, 1));
    }
    reject(&SOURCE.replace("let d = false;", "const d: bool = false;"));
    // An exit-only write must not become part of the original certificate.
    let extra = SOURCE
        .replace("let d = false;", "let d = false; let extra = true;")
        .replacen(PREFIX, "if inner { let extra = false; }", 1);
    reject(&extra);
}

#[test]
fn return_exit_prefix_does_not_export_locals_or_discover_deeper_returns() {
    reject(&SOURCE.replacen(
        PREFIX,
        "if inner { let hidden = local; } let local = hidden;",
        1,
    ));
    reject(&SOURCE.replacen(PREFIX, "let local = future; let future = 1;", 1));
    reject(&SOURCE.replacen(PREFIX, "let hidden = local;", 1).replace(
        "let finished = tail_work(local",
        "let finished = tail_work(hidden",
    ));
    reject(&SOURCE.replacen(PREFIX, "if inner { if take { return d; } }", 1));
}

#[test]
fn return_exit_prefix_still_accepts_single_sided_and_direct_exits() {
    let start = SOURCE.find("                if exit_condition").unwrap();
    let end = start
        + SOURCE[start..]
            .find("                let local = tail_owner")
            .unwrap();
    let one_sided = format!("{}{}", &SOURCE[..start], &SOURCE[end..]);
    for source in [
        one_sided.clone(),
        one_sided.replace("select(pick,", "select(pick != true,"),
    ] {
        let (module, generated) = rewrite(&source);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn return_exit_prefix_retains_five_field_predecessor_authority() {
    let source = include_str!("../../../tests/native_application_bridge/return_paired_in_fields.ns")
        .replace("return d && answer_work(local, seed, 1 / answer);",
            "if d { let a = small_work(a, 1 / answer); let d = bool_work(d, 1 / answer); } let local = initialize(local, 1 / answer); return d && answer_work(local, seed, 1 / answer);");
    let (module, generated) = rewrite(&source);
    assert!(!generated.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert!(module
        .structs
        .iter()
        .filter(|s| s.name.starts_with("__nuis_effect_fields_value"))
        .all(|s| s.fields.len() == 6));
}

#[test]
fn return_exit_prefix_inherits_node_expression_logical_and_depth_limits() {
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
    let edges = 32 - full.logical_edges;
    assert_eq!(nodes, 24);
    assert_eq!(edges, 1);
    assert!(
        !result.inputs.contains("local")
            && !result.inputs.contains("a")
            && !result.inputs.contains("d")
    );
    for mut limited in [
        budget(nodes - 1, 256, 32),
        budget(64, expressions - 1, 32),
        budget(64, 256, edges - 1),
    ] {
        assert!(prove(then_body, &scope, &signatures, &mut limited, 1).is_none());
    }
    assert!(prove(
        then_body,
        &scope,
        &signatures,
        &mut budget(nodes, expressions, edges),
        4
    )
    .is_some());
    assert!(prove(then_body, &scope, &signatures, &mut budget(64, 256, 32), 5).is_none());
}
