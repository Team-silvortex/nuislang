use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn value_catalog_keeps_transitive_types_effects_cycles_and_buffer_admission_separate() {
    let module = parse_nuis_module(
        "mod cpu Main {
      struct Pair { value: i64, seed: i64 }
      struct Mixed { value: i64, flag: bool }
      struct Nested { value: Pair }
      fn divide(a: i64, b: i64) -> i64 { return a / b; }
      fn pack(a: i64, b: i64) -> Pair { return Pair { value: a, seed: b }; }
      fn choose(flag: bool, a: i64, b: i64) -> Pair {
        if flag { return pack(divide(a, b), b); } return pack(a, b);
      }
      fn project(flag: bool, a: i64, b: i64) -> i64 {
        let record: Pair = choose(flag, a, b);
        if flag { return record.value; } return record.seed;
      }
      fn cycle_a(v: i64) -> Pair { return cycle_b(v); }
      fn cycle_b(v: i64) -> Pair { return cycle_a(v); }
      fn effect(v: i64) -> Pair { print(v); return pack(v, v); }
      fn transitive(v: i64) -> Pair { return effect(v); }
      fn main() -> i64 { return 0; }
    }",
    )
    .unwrap();
    let layouts = layouts(&module);
    assert_eq!(
        layouts.keys().map(String::as_str).collect::<Vec<_>>(),
        ["Pair"]
    );
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let keys = |catalog: &ScalarHelpers| catalog.keys().cloned().collect::<Vec<_>>();
    assert_eq!(
        keys(&catalog),
        ["choose", "divide", "main", "pack", "project"]
    );
    assert_eq!(keys(&scalar_helpers::collect(&module)), ["divide", "main"]);
    let mut reversed = module.clone();
    reversed.functions.reverse();
    reversed.structs.reverse();
    assert_eq!(
        keys(&catalog),
        keys(&scalar_helpers::collect_with_layouts(&reversed, &layouts))
    );
    let ty = scalar_type("Pair");
    let mut invalid = zero_value(&ty, &layouts);
    let NirExpr::StructLiteral { fields, .. } = &mut invalid else {
        unreachable!()
    };
    fields[1].0 = fields[0].0.clone();
    assert!(value_type(&invalid, &Scope::new(), &catalog, &layouts).is_none());
    assert!(binary_type(NirBinaryOp::Eq, ty.clone(), ty).is_none());
}

#[test]
fn aggregate_control_outlining_shares_suffixes_and_captures_existing_records() {
    let mut module = parse_nuis_module(
        "mod cpu Main {
      struct Pair { value: i64, seed: i64 }
      fn candidate(v: i64, saved: Pair) -> Pair { return saved; }
      fn main() -> i64 { return 0; }
    }",
    )
    .unwrap();
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "candidate")
        .unwrap();
    let mut body = Vec::new();
    for index in 0..32 {
        body.push(NirStmt::If {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Gt,
                lhs: Box::new(NirExpr::Var("v".into())),
                rhs: Box::new(NirExpr::Int(index)),
            },
            then_body: vec![NirStmt::Let {
                name: "discarded".into(),
                ty: None,
                value: NirExpr::StructLiteral {
                    type_name: "Pair".into(),
                    type_args: vec![],
                    fields: vec![
                        (
                            "value".into(),
                            NirExpr::Binary {
                                op: NirBinaryOp::Div,
                                lhs: Box::new(NirExpr::Int(1)),
                                rhs: Box::new(NirExpr::Var("v".into())),
                            },
                        ),
                        (
                            "seed".into(),
                            NirExpr::FieldAccess {
                                base: Box::new(NirExpr::Var("saved".into())),
                                field: "seed".into(),
                            },
                        ),
                    ],
                },
            }],
            else_body: vec![],
        });
    }
    body.push(NirStmt::Return(Some(NirExpr::Var("saved".into()))));
    function.body = body;
    let layouts = layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    assert!(catalog.contains_key("candidate"));
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let mut helpers = Vec::new();
    let mut guarded = BTreeSet::new();
    scalar_control::outline(
        &mut module,
        &BTreeSet::from(["candidate".into()]),
        &mut names,
        &mut helpers,
        &mut guarded,
        &catalog,
        &layouts,
    );
    assert_eq!(helpers.len(), 32 * 3);
    assert_eq!(guarded.len(), 32 * 2);
    assert!(helpers.iter().all(|f| f.params.len() <= 3));
    assert!(helpers.iter().map(|f| f.body.len()).sum::<usize>() < 32 * 12);
    module.functions.extend(helpers);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}
