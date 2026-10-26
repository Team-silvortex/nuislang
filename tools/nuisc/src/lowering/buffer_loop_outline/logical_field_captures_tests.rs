use super::*;

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}
fn projection(root: &str, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var(root.into())),
        field: field.into(),
    }
}

#[test]
fn lazy_field_captures_keep_all_scalar_kinds_order_dedup_and_hygiene() {
    let kinds = ["bool", "i32", "i64", "f32", "f64"];
    let layouts = BTreeMap::from([(
        "All".into(),
        kinds
            .iter()
            .enumerate()
            .map(|(i, ty)| (format!("f{i}"), scalar_type(ty)))
            .collect(),
    )]);
    let scope = Scope::from([
        ("live".into(), scalar_type("All")),
        ("__nuis_predicate_field".into(), scalar_type("i64")),
    ]);
    let mut args = (0..5)
        .map(|i| projection("live", &format!("f{i}")))
        .collect::<Vec<_>>();
    args.extend([
        projection("live", "f2"),
        NirExpr::Var("__nuis_predicate_field".into()),
    ]);
    let value = NirExpr::Call {
        callee: "reader".into(),
        args: args.clone(),
    };
    let captured = collect(&value, &scope, &layouts).unwrap();
    assert_eq!(
        captured.args,
        [args[..5].to_vec(), vec![args[6].clone()]].concat()
    );
    assert_eq!(
        captured
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["bool", "i32", "i64", "f32", "f64", "i64"]
    );
    let NirExpr::Call {
        args: rewritten, ..
    } = captured.value
    else {
        panic!()
    };
    assert_eq!(rewritten[2], rewritten[5]);
    assert!(captured.params[..5]
        .iter()
        .all(|p| p.name != "__nuis_predicate_field"));
    assert_eq!(
        captured
            .params
            .iter()
            .map(|p| &p.name)
            .collect::<BTreeSet<_>>()
            .len(),
        6
    );
    assert_eq!(
        value,
        NirExpr::Call {
            callee: "reader".into(),
            args
        }
    );
}

#[test]
fn lazy_field_captures_bound_complete_union_and_reject_unsafe_or_unknown_paths() {
    let scope = (0..32)
        .map(|i| (format!("v{i}"), scalar_type("i64")))
        .collect::<Scope>();
    let layouts = BTreeMap::new();
    let call = |n| NirExpr::Call {
        callee: "reader".into(),
        args: (0..n).map(|i| NirExpr::Var(format!("v{i}"))).collect(),
    };
    assert_eq!(
        collect(&call(31), &scope, &layouts).unwrap().params.len(),
        31
    );
    assert!(collect(&call(32), &scope, &layouts).is_none());
    let mut repeated = call(31);
    let NirExpr::Call { args, .. } = &mut repeated else {
        panic!()
    };
    args.extend(vec![NirExpr::Var("v0".into()); 100]);
    assert_eq!(
        collect(&repeated, &scope, &layouts).unwrap().params.len(),
        31
    );
    let mut deep = NirExpr::Int(1);
    for _ in 0..65 {
        deep = binary(NirBinaryOp::Add, deep, NirExpr::Int(1));
    }
    assert!(collect(&deep, &scope, &layouts).is_none());
    let mut wide = NirExpr::Int(1);
    for _ in 0..12 {
        wide = binary(NirBinaryOp::Add, wide.clone(), wide);
    }
    assert!(collect(&wide, &scope, &layouts).is_none());
    let mut reference = scalar_type("Record");
    reference.is_ref = true;
    let scope = Scope::from([
        ("record".into(), scalar_type("Record")),
        ("reference".into(), reference),
    ]);
    let layouts = BTreeMap::from([("Record".into(), vec![("value".into(), scalar_type("i64"))])]);
    for bad in [
        NirExpr::Var("record".into()),
        projection("reference", "value"),
        projection("record", "missing"),
        NirExpr::FieldAccess {
            base: Box::new(call(0)),
            field: "value".into(),
        },
    ] {
        let before = bad.clone();
        assert!(collect(&bad, &scope, &layouts).is_none());
        assert_eq!(before, bad);
    }
}

#[test]
fn lazy_field_capture_helpers_keep_checks_after_guard_even_when_all_fields_are_used() {
    let mut module=crate::frontend::parse_nuis_module("mod cpu Main{struct Pair{tag:i64,flag:bool} fn probe(live:Pair,gate:bool,divisor:i64)->bool{let result=gate&&(live.tag/divisor>0&&live.flag);return result;}fn main()->i64{return 0;}}").unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "probe")
        .unwrap();
    let mut helpers = Vec::new();
    let mut guarded = BTreeSet::new();
    super::super::outline(
        function,
        &mut BTreeSet::new(),
        &mut helpers,
        &mut guarded,
        &catalog,
        &layouts,
    );
    assert!(!helpers.is_empty());
    for helper in &helpers {
        assert!(guarded.contains(&helper.name));
        assert!(helper
            .params
            .iter()
            .all(|p| matches!(p.ty.name.as_str(), "bool" | "i64")));
        assert!(matches!(helper.body.first(), Some(NirStmt::If { .. })));
        let mut inputs = BTreeSet::new();
        control_values::collect_inputs(
            match helper.body.last().unwrap() {
                NirStmt::Return(Some(v)) => v,
                _ => panic!(),
            },
            &mut inputs,
        );
        assert!(!inputs.contains("live"));
    }
    module.functions.extend(helpers);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}
