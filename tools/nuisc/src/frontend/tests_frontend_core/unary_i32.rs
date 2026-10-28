use super::*;

fn operand(value: &NirExpr) -> &NirExpr {
    let NirExpr::CastI64ToI32(word) = value else {
        panic!("not a wrapping i32 negation: {value:?}")
    };
    let NirExpr::Binary {
        op: NirBinaryOp::Sub,
        lhs,
        rhs,
    } = word.as_ref()
    else {
        panic!("subtraction")
    };
    assert_eq!(lhs.as_ref(), &NirExpr::Int(0));
    let NirExpr::CastI32ToI64(value) = rhs.as_ref() else {
        panic!("sign extension")
    };
    value
}
fn printed(source: &str) -> Vec<i64> {
    let yir = crate::pipeline::compile_source(source).unwrap().yir;
    yir_lower_llvm::emit_module(&yir).unwrap();
    yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .unwrap()
    .events
    .iter()
    .filter(|e| e.contains("cpu.print"))
    .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
    .collect()
}

#[test]
fn i32_negation_source_uses_once_only_wide_subtract_and_wrap() {
    let module=parse_nuis_module("mod cpu Main{@noinline fn sample(value:i32)->i32{return value;}fn negative(value:i32)->i32{return -value;}fn called(value:i32)->i32{return -sample(value);}fn nested(value:i32)->i32{return -(-value);}fn computed(value:i32,other:i32)->i32{return -(value+other);}fn main()->i64{return 0;}}").unwrap();
    for name in ["negative", "called", "nested", "computed"] {
        let function = module.functions.iter().find(|f| f.name == name).unwrap();
        assert_eq!(
            function.return_type.as_ref().unwrap(),
            &crate::frontend::i32_type()
        );
        let NirStmt::Return(Some(value)) = &function.body[0] else {
            panic!("return")
        };
        let value = operand(value);
        match name {
            "negative" => assert_eq!(value, &NirExpr::Var("value".into())),
            "called" => assert!(
                matches!(value,NirExpr::Call{callee,args} if callee=="sample"&&args==&vec![NirExpr::Var("value".into())])
            ),
            "nested" => assert_eq!(operand(value), &NirExpr::Var("value".into())),
            "computed" => assert!(matches!(value, NirExpr::CastI64ToI32(_))),
            _ => unreachable!(),
        }
    }
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn i32_negation_source_wraps_minimum_without_changing_other_numeric_profiles() {
    for value in [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX] {
        let expected = value.wrapping_neg();
        let source=format!("mod cpu Main{{@noinline fn negative(value:i32)->i32{{return -value;}}fn main()->i64{{let value=negative(i32_from_i64({value}));if value==i32_from_i64({expected}){{print(1);}}else{{print(0);}}if -(-value)==value{{print(1);}}else{{print(0);}}return 0;}}}}");
        assert_eq!(printed(&source), [1, 1]);
    }
    assert_eq!(printed("mod cpu Main{fn main()->i64{print(-7);if -1.5<0.0{print(1);}else{print(0);}let a:f32=-0.0;if a==0.0{print(1);}else{print(0);}return 0;}}"),[-7,1,1]);
}

#[test]
fn i32_negation_source_generic_lambda_and_exact_kind_rules_agree() {
    let source="mod cpu Main{trait Negatable{fn neg(value:Self)->Self;}impl Negatable for i32{fn neg(value:i32)->i32{return value;}}fn flip<T:Negatable>(value:T)->T{return -value;}fn apply(value:i32,f:Fn1<i32,i32>)->i32{return f(value);}fn main()->i64{let value=i32_from_i64(7);let a=flip(value);let b=apply(value,|x:i32|{let local=-x;return local;});if a==i32_from_i64(-7)&&b==i32_from_i64(-7){print(1);}else{print(0);}return 0;}}";
    assert_eq!(printed(source), [1]);
    for ty in ["bool", "i32?", "ref i32", "i32<i64>"] {
        assert!(parse_nuis_module(&format!("mod cpu Main{{fn wrong(value:{ty})->{ty}{{return -value;}}fn main()->i64{{return 0;}}}}" )).is_err(),"{ty}");
    }
}

#[test]
fn if_print_frontier_preserves_source_order_before_and_after_both_arms() {
    for gate in [false, true] {
        let source = format!("mod cpu Main{{@noinline fn gate(value:bool)->bool{{return value;}}fn main()->i64{{print(-7);if gate({gate}){{print(11);}}else{{print(22);}}print(33);return 0;}}}}");
        assert_eq!(printed(&source), [-7, if gate { 11 } else { 22 }, 33]);
        let yir = crate::pipeline::compile_source(&source).unwrap().yir;
        let prints = yir
            .nodes
            .iter()
            .filter(|n| n.op.module == "cpu" && n.op.instruction == "print")
            .map(|n| n.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(prints.len(), 3);
        for pair in prints.windows(2) {
            let mut reached = std::collections::BTreeSet::from([pair[0].to_owned()]);
            loop {
                let before = reached.len();
                for edge in &yir.edges {
                    if matches!(edge.kind, yir_core::EdgeKind::Effect)
                        && reached.contains(&edge.from)
                    {
                        reached.insert(edge.to.clone());
                    }
                }
                if reached.len() == before {
                    break;
                }
            }
            assert!(
                reached.contains(pair[1]),
                "missing source-ordered effect path: {pair:?}"
            );
        }
    }
}
