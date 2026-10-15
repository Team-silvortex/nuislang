use super::*;
use crate::frontend::parse_nuis_module;

#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/return_typed_exit_prefix_cases.rs"]
mod cases;

const SOURCE: &str =
    include_str!("../../../tests/native_application_bridge/return_plain_exit_prefix.ns");
const RETURN: &str = "return answer_work(local, a, b, c, d, e, 1 / answer);";

fn rewrite(source: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

#[test]
fn return_plain_prefix_calls_literals_and_names_use_the_typed_route() {
    assert!(!SOURCE.contains("&& true"));
    let variants = cases::source_variants();
    assert_eq!(variants.iter().collect::<BTreeSet<_>>().len(), 4);
    assert_eq!(variants.iter().filter(|s| s.contains("&& true")).count(), 2);
    for result in [RETURN, "return true;", "return false;", "return d;"] {
        let at = SOURCE.rfind(RETURN).unwrap();
        let mut source = SOURCE.to_owned();
        source.replace_range(at..at + RETURN.len(), result);
        let (module, names) = rewrite(&source);
        assert!(!names.is_empty(), "{result}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert!(module
            .functions
            .iter()
            .filter(|f| names.contains(&f.name))
            .all(|f| f.params.len() <= 32));
        for callee in ["select", "tail_work"] {
            assert_eq!(
                module
                    .functions
                    .iter()
                    .flat_map(|f| &f.body)
                    .filter(
                        |s| matches!(s, NirStmt::Let {value: NirExpr::Call {callee: c, ..}, ..}
                | NirStmt::Return(Some(NirExpr::Call {callee: c, ..})) if c == callee)
                    )
                    .count(),
                1
            );
        }
        assert!(module
            .structs
            .iter()
            .filter(|s| s.name.starts_with("__nuis_effect_fields_value"))
            .all(|s| s
                .fields
                .iter()
                .map(|f| f.ty.name.as_str())
                .collect::<Vec<_>>()
                == ["i64", "i32", "f32", "f64", "bool", "bool"]));
    }
    // This was rejected only by the old logical-return syntax gate.
    for body in [
        "let local = decide(saved) && first; return local;",
        "let local = saved; if decide(first) { let local = first; } return local;",
    ] {
        let source = format!(
            "mod cpu Main {{
        fn decide(v: bool) -> bool {{ print(1); return v; }}
        fn observe(gate: bool, saved: bool, first: bool) -> bool {{
            if gate {{ {body} }} else {{ return saved; }}
        }} fn main() -> i64 {{ return 0; }} }}"
        );
        let (module, names) = rewrite(&source);
        assert!(!names.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

fn reject(mut module: NirModule) {
    let original = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let original_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, original);
    assert_eq!(names, original_names);
}

#[test]
fn return_plain_prefix_non_bool_and_authority_vetoes_are_atomic() {
    for value in [
        NirExpr::Int(0),
        NirExpr::Var("local".into()),
        NirExpr::Call {
            callee: "tail_work".into(),
            args: vec![NirExpr::Var("local".into()), NirExpr::Int(1)],
        },
        NirExpr::Call {
            callee: "missing".into(),
            args: vec![],
        },
    ] {
        let mut module = parse_nuis_module(SOURCE).unwrap();
        let f = module
            .functions
            .iter_mut()
            .find(|f| f.name == "observe")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut f.body[0] else {
            panic!()
        };
        *then_body.last_mut().unwrap() = NirStmt::Return(Some(value));
        reject(module);
    }
    for source in [
        SOURCE.replacen(
            "let local = exit_owner(local, 1 / yes_owner);",
            "let seed = exit_owner(local, 1 / yes_owner);",
            1,
        ),
        SOURCE.replace("let d = flag_seed;", "const d: bool = flag_seed;"),
        SOURCE.replacen(
            "let local = exit_owner(local, 1 / yes_owner);",
            "let a = exit_small(a, 1 / small_proof);",
            1,
        ),
        SOURCE.replacen(
            "if inner_condition(inner, 1 / choice)",
            "if inner { return d; } if inner_condition(inner, 1 / choice)",
            1,
        ),
    ] {
        reject(parse_nuis_module(&source).unwrap());
    }
    let mut module = parse_nuis_module(SOURCE).unwrap();
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "observe")
        .unwrap()
        .return_type = Some(scalar_type("i64"));
    reject(module);
}

#[test]
fn return_plain_prefix_exact_budget_and_depth_need_no_logical_edge() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let f = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let scope = f
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
    let NirStmt::If { then_body, .. } = &f.body[0] else {
        panic!()
    };
    let budget = |nodes, expressions| nested::Budget {
        nodes,
        expressions,
        logical_edges: 0,
    };
    let mut full = budget(64, 256);
    let value = in_fields::prove(then_body, &scope, &signatures, &mut full, 1).unwrap();
    let used = 256 - full.expressions;
    assert_eq!(64 - full.nodes, 39);
    assert_eq!(full.logical_edges, 0);
    assert!(["local", "a", "b", "c", "d", "e"]
        .iter()
        .all(|n| !value.inputs.contains(*n)));
    assert!(in_fields::prove(then_body, &scope, &signatures, &mut budget(38, 256), 1).is_none());
    assert!(
        in_fields::prove(then_body, &scope, &signatures, &mut budget(39, used - 1), 1).is_none()
    );
    assert!(in_fields::prove(then_body, &scope, &signatures, &mut budget(39, used), 4).is_some());
    assert!(in_fields::prove(then_body, &scope, &signatures, &mut budget(39, used), 5).is_none());
}

#[test]
fn return_plain_prefix_keeps_effect_admission_prefix_requirement_and_capture_cap() {
    for count in [31, 32] {
        let mut scope = Scope::from([("gate".into(), scalar_type("bool"))]);
        let mut args = Vec::new();
        for i in 0..count {
            let name = format!("v{i}");
            scope.insert(name.clone(), scalar_type("bool"));
            args.push(NirExpr::Var(name));
        }
        let signatures = Signatures::from([
            (
                "all".into(),
                (vec![scalar_type("bool"); count], scalar_type("bool")),
            ),
            (
                "touch".into(),
                (vec![scalar_type("bool")], scalar_type("bool")),
            ),
        ]);
        let yes = [
            NirStmt::Let {
                name: "unused".into(),
                ty: None,
                value: NirExpr::Call {
                    callee: "touch".into(),
                    args: vec![NirExpr::Var("v0".into())],
                },
            },
            NirStmt::Return(Some(NirExpr::Call {
                callee: "all".into(),
                args,
            })),
        ];
        let no = [NirStmt::Return(Some(NirExpr::Bool(false)))];
        let condition = NirExpr::Var("gate".into());
        let pure = BTreeSet::from(["all".into()]);
        let plan = prepare(&condition, &yes, &no, &scope, &signatures, &pure);
        assert_eq!(plan.is_some(), count == 31);
        if let Some(plan) = plan {
            assert_eq!(plan.params.len(), 31);
        }
        let pure = BTreeSet::from(["all".into(), "touch".into()]);
        assert!(prepare(&condition, &yes, &no, &scope, &signatures, &pure).is_none());
        assert!(prepare(
            &condition,
            &yes[1..],
            &no,
            &scope,
            &signatures,
            &BTreeSet::new()
        )
        .is_none());
    }
}
