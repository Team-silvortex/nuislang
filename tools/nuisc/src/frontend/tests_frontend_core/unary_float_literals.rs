use super::*;

fn atom(ty: &str, value: &str) -> NirExpr {
    match ty {
        "f32" => NirExpr::F32(value.into()),
        "f64" => NirExpr::F64(value.into()),
        _ => unreachable!(),
    }
}

fn sign_flip_operand<'a>(expr: &'a NirExpr, ty: &str) -> &'a NirExpr {
    let word = match (ty, expr) {
        ("f32", NirExpr::UnpackF32Word(word)) | ("f64", NirExpr::UnpackF64Word(word)) => word,
        _ => panic!("{ty}: {expr:?}"),
    };
    let NirExpr::Binary { op, lhs, rhs } = word.as_ref() else {
        panic!()
    };
    assert_eq!(*op, NirBinaryOp::Xor);
    assert_eq!(
        **rhs,
        NirExpr::Int(if ty == "f32" { 1_i64 << 31 } else { i64::MIN })
    );
    match (ty, lhs.as_ref()) {
        ("f32", NirExpr::PackF32Word(operand)) | ("f64", NirExpr::PackF64Word(operand)) => operand,
        _ => panic!(),
    }
}

#[test]
fn unary_float_literals_preserve_sign_spelling_and_nested_negation() {
    for ty in ["f32", "f64"] {
        for (expression, expected) in [
            ("-0.0", "-0.0"),
            ("-(-0.0)", "0.0"),
            ("-(-(-0.0))", "-0.0"),
            ("-1.2500", "-1.2500"),
            ("-(-2.5000)", "2.5000"),
            ("-(0.0000000001)", "-0.0000000001"),
        ] {
            for binding in ["let", "const"] {
                let source = format!(
                    "mod cpu Main {{ fn main() -> i64 {{
                    {binding} value: {ty} = {expression}; return 0;
                }} }}"
                );
                let module = parse_nuis_module(&source).unwrap();
                let function = &module.functions[0];
                let (NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) = &function.body[0]
                else {
                    panic!("{source}")
                };
                assert_eq!(*value, atom(ty, expected), "{source}");
                crate::nir_verify::verify_nir_module(&module).unwrap();
            }
        }
    }
}

#[test]
fn unary_float_literals_respect_inferred_and_expected_contexts() {
    for ty in ["f32", "f64"] {
        let module = parse_nuis_module(&format!(
            "mod cpu Main {{
            const ZERO: {ty} = 0.0;
            struct State {{ value: {ty} }}
            fn accept(value: {ty}) -> {ty} {{ return value; }}
            fn negative() -> {ty} {{ return -ZERO; }}
            fn state() -> State {{ return State {{ value: -0.0 }}; }}
            fn main() -> i64 {{ let inferred = -0.0;
                let value: {ty} = accept(-0.0); return 0; }}
        }}"
        ))
        .unwrap();
        let negative = module
            .functions
            .iter()
            .find(|f| f.name == "negative")
            .unwrap();
        assert_eq!(negative.body, vec![NirStmt::Return(Some(atom(ty, "-0.0")))]);
        let state = module.functions.iter().find(|f| f.name == "state").unwrap();
        let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) = &state.body[0] else {
            panic!()
        };
        assert_eq!(fields[0].1, atom(ty, "-0.0"));
        let main = module.functions.iter().find(|f| f.name == "main").unwrap();
        assert!(matches!(&main.body[0], NirStmt::Let { value, .. }
            if value == &atom("f64", "-0.0")));
        assert!(matches!(&main.body[1], NirStmt::Let {
            value: NirExpr::Call { args, .. }, .. }
            if args == &vec![atom(ty, "-0.0")]));
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn unary_float_literals_leave_nonliteral_operands_once_only_and_unfolded() {
    for ty in ["f32", "f64"] {
        let module = parse_nuis_module(&format!(
            "mod cpu Main {{
            @noinline fn sample(value: {ty}) -> {ty} {{ return value; }}
            fn negate(value: {ty}) -> {ty} {{ return -value; }}
            fn computed(value: {ty}) -> {ty} {{ return -(value + 1.25); }}
            fn called(value: {ty}) -> {ty} {{ return -sample(value); }}
            fn nested(value: {ty}) -> {ty} {{ return -(-value); }}
            fn main() -> i64 {{ return 0; }}
        }}"
        ))
        .unwrap();
        for name in ["negate", "computed", "called", "nested"] {
            let function = module.functions.iter().find(|f| f.name == name).unwrap();
            let NirStmt::Return(Some(value)) = &function.body[0] else {
                panic!()
            };
            let operand = sign_flip_operand(value, ty);
            match name {
                "negate" => assert_eq!(*operand, NirExpr::Var("value".into())),
                "computed" => assert!(matches!(
                    operand,
                    NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        ..
                    }
                )),
                "called" => assert!(matches!(operand, NirExpr::Call { callee, args }
                    if callee == "sample" && args == &vec![NirExpr::Var("value".into())])),
                "nested" => assert_eq!(
                    *sign_flip_operand(operand, ty),
                    NirExpr::Var("value".into())
                ),
                _ => unreachable!(),
            }
        }
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn unary_float_literals_do_not_bypass_expected_type_validation() {
    for ty in ["bool", "i64", "i32", "f32?", "ref f64"] {
        let source = format!(
            "mod cpu Main {{ fn main() -> i64 {{
            let value: {ty} = -0.0; return 0;
        }} }}"
        );
        let error = parse_nuis_module(&source).unwrap_err();
        assert!(
            error.contains("float literal") && error.contains("expected type"),
            "{ty}: {error}"
        );
    }
}
