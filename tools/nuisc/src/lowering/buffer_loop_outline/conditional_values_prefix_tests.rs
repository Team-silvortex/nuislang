use super::*;

const SOURCE: &str = "mod cpu Main {
    struct Packet { unused: i64, value: i64 }
    fn produce(value: i64, divisor: i64) -> Packet {
        return Packet { unused: 20 / divisor, value: value };
    }
    fn entry(gate: bool, divisor: i64) -> i64 {
        let result: i64 = 7;
        if gate {
            let first: i64 = 10 / divisor;
            const saved: Packet = produce(first, divisor);
            let result: i64 = saved.value;
        }
        print(result); return result;
    }
    fn main() -> i64 { return entry(false, 0) + entry(true, 2); }
}";

fn outlined(module: &mut NirModule) -> BTreeSet<String> {
    let layouts = control_values::TypedLayouts::collect(module);
    let carries = control_values::layouts(module);
    let control = scalar_helpers::collect_with_layouts(module, &carries);
    let catalog = scalar_helpers::collect_typed_values(module, &layouts, &control);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    outline(module, &catalog, &BTreeSet::new(), &layouts, &mut names)
}

fn entry(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "entry")
        .unwrap()
}

#[test]
fn conditional_prefix_values_keep_fresh_ordered_locals_inside_guarded_selections() {
    let mut module = crate::frontend::parse_nuis_module(SOURCE).unwrap();
    let original = entry(&mut module).body.clone();
    let generated = outlined(&mut module);
    assert_eq!(generated.len(), 1);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let selection = module
        .functions
        .iter()
        .find(|f| generated.contains(&f.name))
        .unwrap();
    assert!(selection
        .params
        .iter()
        .all(|p| !["first", "saved"].contains(&p.name.as_str())));
    let NirStmt::If { then_body, .. } = &selection.body[0] else {
        panic!()
    };
    let NirStmt::If {
        then_body: original_body,
        ..
    } = &original[1]
    else {
        panic!()
    };
    assert_eq!(&then_body[..2], &original_body[..2]);
    assert!(matches!(
        then_body.last(),
        Some(NirStmt::Return(Some(NirExpr::FieldAccess { .. })))
    ));
    let compiled = crate::pipeline::compile_source(SOURCE).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let prints = trace
        .events
        .iter()
        .filter(|e| e.contains("cpu.print"))
        .collect::<Vec<_>>();
    assert_eq!(prints.len(), 2);
    for (line, value) in prints.iter().zip([7, 5]) {
        assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
    }
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
}

#[test]
fn conditional_prefix_values_gate_pure_call_conditions_without_eager_rhs_checks() {
    for (op, gate, divisor, expected) in [
        ("&&", false, 0, Some(19)),
        ("||", true, 0, Some(11)),
        ("&&", true, 2, Some(11)),
        ("||", false, 2, Some(11)),
        ("&&", true, -2, Some(19)),
        ("||", false, -2, Some(19)),
        ("&&", true, 0, None),
        ("||", false, 0, None),
    ] {
        let source = format!("mod cpu Main {{
            struct Packet {{ unused: i64, value: i64 }}
            fn produce(divisor: i64) -> Packet {{ return Packet {{ unused: 10 / divisor, value: divisor }}; }}
            fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
            fn entry(gate: bool, divisor: i64) -> i64 {{ if gate {op} helper(produce(divisor)) {{ return 11; }} return 19; }}
            fn main() -> i64 {{ return entry({gate}, {divisor}); }}
        }}");
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        let result = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&compiled.yir),
            &yir_verify::default_registry(),
        );
        if let Some(expected) = expected {
            let trace = result.unwrap();
            let main = compiled
                .yir
                .functions
                .iter()
                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                .unwrap();
            assert_eq!(
                trace.values[&main.result.as_ref().unwrap().node],
                yir_core::Value::Int(expected)
            );
        } else {
            let error = result.unwrap_err();
            assert!(error.contains("zero"), "{error}");
        }
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
}

#[test]
fn conditional_prefix_values_keep_rebindings_effects_kinds_and_limits_conservative() {
    for mutation in 0..9 {
        let mut module = crate::frontend::parse_nuis_module(SOURCE).unwrap();
        let NirStmt::If { then_body, .. } = &mut entry(&mut module).body[1] else {
            panic!()
        };
        match mutation {
            0 => {
                let NirStmt::Let { name, .. } = &mut then_body[0] else {
                    panic!()
                };
                *name = "result".into();
            }
            1 => {
                let NirStmt::Const { name, .. } = &mut then_body[1] else {
                    panic!()
                };
                *name = "first".into();
            }
            2 => {
                let NirStmt::Let { value, .. } = &mut then_body[0] else {
                    panic!()
                };
                *value = NirExpr::Var("saved".into());
            }
            3 => {
                let NirStmt::Let { ty, .. } = &mut then_body[0] else {
                    panic!()
                };
                ty.as_mut().unwrap().is_ref = true;
            }
            4 => then_body.insert(0, NirStmt::Print(NirExpr::Int(1))),
            5 => {
                let last = then_body.pop().unwrap();
                for index in 0..32 {
                    then_body.push(NirStmt::Let {
                        name: format!("local_{index}"),
                        ty: Some(scalar_type("i64")),
                        value: NirExpr::Int(1),
                    });
                }
                then_body.push(last);
            }
            6 => {
                let NirStmt::Let { value, .. } = &mut then_body[0] else {
                    panic!()
                };
                for _ in 0..70 {
                    *value = NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(value.clone()),
                        rhs: Box::new(NirExpr::Int(0)),
                    };
                }
            }
            7 => {
                let NirStmt::Let { value, .. } = &mut then_body[0] else {
                    panic!()
                };
                *value = NirExpr::Int(1);
                for _ in 0..13 {
                    *value = NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(value.clone()),
                        rhs: Box::new(value.clone()),
                    };
                }
            }
            8 => module
                .functions
                .iter_mut()
                .find(|f| f.name == "produce")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1))),
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(outlined(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}
