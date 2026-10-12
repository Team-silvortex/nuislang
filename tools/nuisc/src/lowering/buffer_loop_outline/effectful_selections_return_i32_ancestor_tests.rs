use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        fn decide(value: bool) -> bool {{ print(98); return value; }}
        fn boolean(value: bool) -> bool {{ return value; }}
        fn relay(value: i32) -> i32 {{ print(96); return value; }}
        fn observe(gate: bool, first: bool, saved: bool, seed: i64) -> bool {{
            print(99); {body} print(80); return false;
        }} fn main() -> i64 {{ return 0; }}
    }}"
    )
}
fn parsed(body: &str) -> NirModule {
    parse_nuis_module(&source(body)).unwrap()
}
fn rewrite(body: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parsed(body);
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}
fn reject(mut module: NirModule) {
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}
fn reject_source(body: &str) {
    match parse_nuis_module(&source(body)) {
        Ok(module) => reject(module),
        Err(error) => assert!(error.contains("unknown value"), "{error}"),
    }
}
fn body(selected: &str) -> String {
    format!(
        "if decide(gate) {{
        let local = i32_from_i64(seed);
        if first {{ let child = saved; {selected} }}
        else {{ let local = relay(local); }}
        return (local == i32_from_i64(seed)) && decide(first);
    }} else {{ return saved; }}"
    )
}

#[test]
fn return_i32_ancestor_private_bool_accepts_orders_retention_nested_sequences_and_one_typed_target()
{
    for selected in [
        "if saved {let local = relay(local);let child = child || decide(first);} else {let child = child && first;let local = relay(local);}",
        "if saved {let child = child || decide(first);let local = relay(local);} else {let local = relay(local);}",
        "if saved {let local = relay(local);let child = child && first;if child {let local = relay(local);let child = decide(first);} let local = relay(local);} else {let child = decide(first);let local = relay(local);} if child {let local = relay(local);} let ignored = relay(local);",
    ] {
        for inverted in [false, true] {
            let text = body(selected);
            let text = if inverted { text.replace("if decide(gate)", "if decide(gate == false)") } else { text };
            let (module, generated) = rewrite(&text);
            assert!(!generated.is_empty(), "{selected}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert!(module.functions.iter().filter(|f| generated.contains(&f.name))
                .all(|f| f.params.len() <= 32 && matches!(f.body.first(), Some(NirStmt::If {..}))));
            assert!(module.functions.iter().any(|f| generated.contains(&f.name)
                && f.return_type == Some(scalar_type("i32"))));
            let observe = module.functions.iter().find(|f| f.name == "observe").unwrap();
            for statement in &observe.body {
                if let NirStmt::Let {value: NirExpr::Call {callee,..},..} = statement {
                    let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                    assert_eq!(helper.return_type, Some(scalar_type("bool")));
                    assert!(!helper.params.iter().any(|p| ["local","child","ignored"].contains(&p.name.as_str())));
                }
            }
        }
    }
}

#[test]
fn return_i32_ancestor_private_bool_rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically(
) {
    for selected in [
        "if saved {let local = relay(local);let child = decide(first);let saved = first;}",
        "let sibling = first;if saved {let local = relay(local);let child = decide(first);let sibling = false;}",
        "const child: bool = saved || false;if saved {let local = relay(local);let child = first;}",
        "if saved {let local = relay(local);const child: bool = child || false;}",
        "if saved {let local: bool = first;let child = decide(first);}",
        "if saved {let local = relay(local);let child = boolean(child || first);}",
        "if saved {let local = relay(local);let child = first;print(94);}",
        "if saved {let local = relay(local);return child;} else {let child = first;}",
        "if saved {let local = relay(local);let child = first;} let late = missing;",
    ] { reject_source(&body(selected)); }
    for text in [
        "if gate {const local: f64 = 0.0;if first {let child = saved;if saved {let local: f64 = 0.0;let child = decide(first);}} return (local == 0.0) && decide(first);} else {return saved;}",
        "if gate {let local = i32_from_i64(seed);if first {const child: i32 = i32_from_i64(0);if saved {let local = relay(local);let child = relay(child);}} return (local == i32_from_i64(seed)) && decide(first);} else {return saved;}",
        "if gate {let local = i32_from_i64(seed);if first {let child = saved;if saved {let local = relay(local);let child = first;}} else {let local = child;} return (local == i32_from_i64(seed)) && decide(first);} else {return saved;}",
    ] { reject_source(text); }
    for kind in ["ref", "optional", "generic", "u32", "String"] {
        let mut ty = scalar_type("i32");
        match kind {
            "ref" => ty.is_ref = true,
            "optional" => ty.is_optional = true,
            "generic" => ty.generic_args.push(scalar_type("i32")),
            other => ty = scalar_type(other),
        }
        assert!(!owner_type(&ty), "{kind}");
    }
}

fn eval(expr: &NirExpr, values: &BTreeMap<String, i64>) -> i64 {
    let recurse = |v| eval(v, values);
    match expr {
        NirExpr::Int(v) => *v,
        NirExpr::Bool(v) => i64::from(*v),
        NirExpr::Var(n) => values[n],
        NirExpr::CastBoolToI64(v) => {
            let n = recurse(v);
            assert!((0..=1).contains(&n));
            n
        }
        NirExpr::CastI32ToI64(v) | NirExpr::CastI64ToI32(v) => {
            i64::from(i32::try_from(recurse(v)).unwrap())
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let (a, b) = (i128::from(recurse(lhs)), i128::from(recurse(rhs)));
            i64::try_from(match op {
                NirBinaryOp::Add => a + b,
                NirBinaryOp::Sub => a - b,
                NirBinaryOp::Mul => a * b,
                NirBinaryOp::Ge => i128::from(a >= b),
                _ => panic!("unexpected or trapping codec operation {op:?}"),
            })
            .unwrap()
        }
        _ => panic!("unexpected codec {expr:?}"),
    }
}

#[test]
fn return_i32_ancestor_private_bool_codec_roundtrips_signed_extremes_and_both_flags_without_traps()
{
    let scope = Scope::from([
        ("local".into(), scalar_type("i32")),
        ("child".into(), scalar_type("bool")),
    ]);
    let stage = || regions::Staging {
        local: scope.clone(),
        defined: BTreeSet::new(),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: vec![],
    };
    let (selection, reads, _) = transport::prepare(
        predicates::Predicate::Scalar(NirExpr::Bool(true)),
        stage(),
        stage(),
        &scope,
        "local",
        "child",
    )
    .unwrap();
    assert_eq!(reads, BTreeSet::from(["local".into(), "child".into()]));
    let mut bindings = BTreeSet::from(["__nuis_effect_call_pair_0".into()]);
    let mut helpers = vec![];
    let output = transport::install(
        selection,
        &mut BTreeSet::new(),
        &mut bindings,
        &mut helpers,
        &mut vec![],
    );
    let NirStmt::Return(Some(pack)) = helpers[0].body.last().unwrap() else {
        unreachable!()
    };
    let carrier = bindings
        .iter()
        .find(|n| {
            n.starts_with("__nuis_effect_call_pair") && n.as_str() != "__nuis_effect_call_pair_0"
        })
        .unwrap();
    for value in [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX] {
        for flag in [false, true] {
            let mut values = BTreeMap::from([
                ("local".into(), i64::from(value)),
                ("child".into(), i64::from(flag)),
            ]);
            let word = eval(pack, &values);
            assert_eq!(
                word,
                i64::from(value) + 2_147_483_648 + i64::from(flag) * 4_294_967_296
            );
            assert!((0..=8_589_934_591).contains(&word));
            values.insert(carrier.clone(), word);
            for statement in &output[output.len() - 2..] {
                let NirStmt::Let { name, value, .. } = statement else {
                    unreachable!()
                };
                values.insert(name.clone(), eval(value, &values));
            }
            assert_eq!(
                (values["local"], values["child"]),
                (i64::from(value), i64::from(flag))
            );
        }
    }
    assert!(
        matches!(output.last(),Some(NirStmt::Let {name,ty:Some(ty),value:NirExpr::CastI64ToI32(_)})
        if name=="local" && ty==&scalar_type("i32"))
    );
}

#[test]
fn return_i32_ancestor_private_bool_shared_original_budget_and_capture_boundary() {
    let scope = Scope::from([
        ("local".into(), scalar_type("i32")),
        ("seed".into(), scalar_type("i32")),
        ("child".into(), scalar_type("bool")),
        ("first".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("bool")),
    ]);
    let let_var = |name: &str, value: &str| NirStmt::Let {
        name: name.into(),
        ty: None,
        value: NirExpr::Var(value.into()),
    };
    let yes = vec![let_var("local", "seed"), let_var("child", "saved")];
    let suffix = vec![
        NirStmt::If {
            condition: NirExpr::Var("child".into()),
            then_body: vec![let_var("child", "first"), let_var("local", "seed")],
            else_body: vec![],
        },
        let_var("local", "seed"),
    ];
    let stage = || regions::Staging {
        local: scope.clone(),
        defined: BTreeSet::from(["child".into()]),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: vec![],
    };
    let mut entering = scope.clone();
    entering.remove("child");
    let owner = Target("local");
    let child = mixed::classify(&yes, &[], &entering, &stage(), &owner, 0)
        .unwrap()
        .unwrap();
    let default = || nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    };
    let proof = |yes: &[NirStmt], tail: &[NirStmt], budget: &mut nested::Budget| {
        mixed::continue_flat(
            &NirExpr::Var("first".into()),
            yes,
            &[],
            tail,
            &scope,
            &owner,
            &child,
            &BTreeMap::new(),
            budget,
            0,
        )
    };
    for (nodes, expressions, accepted) in [(7, 7, true), (6, 7, false), (7, 6, false)] {
        let mut budget = nested::Budget {
            nodes,
            expressions,
            logical_edges: 0,
        };
        let outcome = proof(&yes, &suffix, &mut budget);
        assert_eq!(outcome.is_some(), accepted);
        if accepted {
            assert_eq!(
                (budget.nodes, budget.expressions, budget.logical_edges),
                (0, 0, 0)
            );
            assert_eq!(outcome.unwrap().ty, scalar_type("i32"));
        }
    }
    for count in [16, 17] {
        let mut arm = yes.clone();
        arm.extend((2..count).map(|_| let_var("local", "seed")));
        assert_eq!(proof(&arm, &[], &mut default()).is_some(), count == 16);
    }
    for count in [31, 32] {
        let mut staged = stage();
        staged.defined.clear();
        for index in 0..count - 2 {
            let name = format!("capture_{index}");
            staged.local.insert(name.clone(), scalar_type("i32"));
            staged.inputs.insert(name);
        }
        let result = transport::prepare(
            predicates::Predicate::Scalar(NirExpr::Bool(true)),
            staged,
            stage(),
            &scope_with_captures(&scope, count - 2),
            "local",
            "child",
        );
        assert_eq!(result.is_some(), count == 31);
    }
    for depth in [7, 8] {
        assert_eq!(
            mixed::continue_flat(
                &NirExpr::Bool(true),
                &yes,
                &[],
                &[],
                &scope,
                &owner,
                &child,
                &BTreeMap::new(),
                &mut default(),
                depth
            )
            .is_some(),
            depth == 7
        );
    }
}

fn scope_with_captures(scope: &Scope, count: usize) -> Scope {
    let mut scope = scope.clone();
    for index in 0..count {
        scope.insert(format!("capture_{index}"), scalar_type("i32"));
    }
    scope
}
