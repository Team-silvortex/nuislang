use super::*;

fn expression(text: &str) -> AstExpr {
    let ast = parse_nuis_ast(&format!(
        "mod cpu Main{{fn main()->bool{{return {text};}}}}"
    ))
    .unwrap();
    let AstStmt::Return(Some(value)) = &ast.functions[0].body[0] else {
        panic!("return expression");
    };
    value.clone()
}

fn print_values(text: &str) -> Vec<i64> {
    let yir = crate::pipeline::compile_source(text).unwrap().yir;
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    trace
        .events
        .iter()
        .filter(|e| e.contains("cpu.print"))
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn bool_xor_source_precedence_association_and_rendering_are_explicit() {
    let expr = expression("a || b && c ^ d == e");
    let AstExpr::Binary {
        op: AstBinaryOp::Or,
        rhs,
        ..
    } = expr
    else {
        panic!("OR");
    };
    let AstExpr::Binary {
        op: AstBinaryOp::And,
        rhs,
        ..
    } = *rhs
    else {
        panic!("AND");
    };
    let AstExpr::Binary {
        op: AstBinaryOp::Eq,
        lhs,
        ..
    } = *rhs
    else {
        panic!("equality");
    };
    assert!(matches!(
        *lhs,
        AstExpr::Binary {
            op: AstBinaryOp::Xor,
            ..
        }
    ));
    let AstExpr::Binary {
        op: AstBinaryOp::Xor,
        lhs,
        rhs,
    } = expression("a ^ b ^ c")
    else {
        panic!("XOR");
    };
    assert!(matches!(
        *lhs,
        AstExpr::Binary {
            op: AstBinaryOp::Xor,
            ..
        }
    ));
    assert_eq!(*rhs, AstExpr::Var("c".into()));
    let AstExpr::Binary {
        op: AstBinaryOp::Xor,
        lhs,
        rhs,
    } = expression("1 + 2 ^ 3 * 4")
    else {
        panic!("arithmetic");
    };
    assert!(matches!(
        *lhs,
        AstExpr::Binary {
            op: AstBinaryOp::Add,
            ..
        }
    ));
    assert!(matches!(
        *rhs,
        AstExpr::Binary {
            op: AstBinaryOp::Mul,
            ..
        }
    ));
    let ast = parse_nuis_ast("mod cpu Main{fn main()->bool{return true ^ false;}}").unwrap();
    assert!(crate::render::render_ast(&ast).contains("true ^ false"));
    let nir = lower_project_ast_to_nir(&ast, &[]).unwrap();
    assert!(crate::render::render_nir(&nir).contains("true ^ false"));
}

#[test]
fn bool_xor_source_truth_table_emits_native_yir_xor() {
    for a in [false, true] {
        for b in [false, true] {
            let text=format!("mod cpu Main{{@noinline fn bits(a:bool,b:bool)->bool{{return a ^ b;}}fn main()->i64{{if bits({a},{b}){{print(1);}}else{{print(0);}}return 0;}}}}");
            let yir = crate::pipeline::compile_source(&text).unwrap().yir;
            assert!(yir
                .nodes
                .iter()
                .any(|n| n.op.module == "cpu" && n.op.instruction == "xor"));
            assert!(yir_lower_llvm::emit_module(&yir)
                .unwrap()
                .contains("xor i1"));
            assert_eq!(print_values(&text), [i64::from(a ^ b)]);
        }
    }
}

#[test]
fn bool_xor_source_keeps_i64_builtin_strict_and_rejects_cross_kinds() {
    assert_eq!(
        print_values("mod cpu Main{fn main()->i64{print(6 ^ 3);print(i64_xor(6,3));return 0;}}"),
        [5, 5]
    );
    for expr in [
        "true ^ 1",
        "1 ^ false",
        "1.0 ^ 2.0",
        "i64_xor(true,false)",
        "i64_xor(1,false)",
        "i32_from_i64(1) ^ i32_from_i64(2)",
    ] {
        let text = format!("mod cpu Main{{fn main()->bool{{return {expr};}}}}");
        assert!(parse_nuis_module(&text).is_err(), "{expr}");
    }
    for text in ["true ^^ false", "true ^= false"] {
        assert!(parse_nuis_ast(&format!(
            "mod cpu Main{{fn main()->bool{{return {text};}}}}"
        ))
        .is_err());
    }
}

#[test]
fn bool_xor_nir_inference_rejects_modified_and_nonprimitive_operands() {
    use crate::frontend::{bool_type, i64_type, infer_nir_expr_type};
    use std::collections::BTreeMap;
    let expr = NirExpr::Binary {
        op: NirBinaryOp::Xor,
        lhs: Box::new(NirExpr::Var("a".into())),
        rhs: Box::new(NirExpr::Var("b".into())),
    };
    for ty in [bool_type(), i64_type()] {
        let check = |left: nuis_semantics::model::NirTypeRef, right| {
            infer_nir_expr_type(
                &expr,
                &BTreeMap::from([("a".into(), left), ("b".into(), right)]),
                &BTreeMap::new(),
                &BTreeMap::new(),
            )
        };
        assert_eq!(check(ty.clone(), ty.clone()), Some(ty.clone()));
        for field in 0..3 {
            let mut altered = ty.clone();
            match field {
                0 => altered.is_ref = true,
                1 => altered.is_optional = true,
                _ => altered.generic_args.push(i64_type()),
            }
            assert_eq!(check(altered.clone(), altered.clone()), None);
            assert_eq!(check(ty.clone(), altered), None);
        }
        for other in [
            crate::frontend::i32_type(),
            crate::frontend::f32_type(),
            crate::frontend::f64_type(),
        ] {
            assert_eq!(check(other.clone(), other), None);
        }
    }
}

#[test]
fn bool_xor_source_generic_and_lambda_inference_agree() {
    let text="mod cpu Main{fn keep<T>(value:T)->T{return value;}fn apply(a:bool,b:bool,f:Fn2<bool,bool,bool>)->bool{return f(a,b);}fn main()->i64{let a=keep(true ^ false);let value=apply(a,false,|x:bool,y:bool|{let flag=x ^ y;return flag;});if value{print(1);}else{print(0);}let bits=keep(6 ^ 3);print(bits);return 0;}}";
    assert_eq!(print_values(text), [1, 5]);
}

#[test]
fn bool_xor_source_caret_survives_stage_neutral_tokens() {
    let text = "mod cpu Main{fn main()->bool{return true ^ false;}}";
    let rendered = crate::frontend::render_stage_neutral_token_stream(text).unwrap();
    assert!(rendered.contains("symbol\t94\n"));
    crate::frontend::verify_stage_neutral_token_stream(text, &rendered).unwrap();
    let tampered = rendered.replace("symbol\t94\n", "symbol\t124\n");
    assert!(crate::frontend::verify_stage_neutral_token_stream(text, &tampered).is_err());
}
