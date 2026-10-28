use super::*;

fn output(source: &str) -> Vec<i64> {
    let yir = crate::pipeline::compile_source(source).unwrap().yir;
    let llvm = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(llvm.contains("sext i32"));
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
fn i32_widening_source_emits_one_exact_signed_extension() {
    let module=parse_nuis_module("mod cpu Main{@noinline fn sample(value:i32)->i32{return value;}fn widen(value:i32)->i64{return i64_from_i32(sample(value));}fn main()->i64{return widen(i32_from_i64(-1));}}").unwrap();
    let function = module.functions.iter().find(|f| f.name == "widen").unwrap();
    assert_eq!(
        function.return_type.as_ref().unwrap(),
        &crate::frontend::i64_type()
    );
    assert!(
        matches!(&function.body[0],NirStmt::Return(Some(NirExpr::CastI32ToI64(operand))) if matches!(operand.as_ref(),NirExpr::Call{callee,args} if callee=="sample"&&args==&vec![NirExpr::Var("value".into())]))
    );
    assert!(crate::render::render_nir(&module).contains("i64_from_i32(sample(value))"));
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn i32_widening_source_preserves_signed_range_and_negation_order() {
    for value in [i32::MIN, i32::MIN + 1, -1, 0, 1, 107, i32::MAX] {
        let source=format!("mod cpu Main{{@noinline fn widen(value:i32)->i64{{return i64_from_i32(value);}}fn main()->i64{{let value=i32_from_i64({value});print(widen(value));print(i64_from_i32(-value));print(-i64_from_i32(value));return 0;}}}}");
        assert_eq!(
            output(&source),
            [
                i64::from(value),
                i64::from(value.wrapping_neg()),
                -i64::from(value)
            ]
        );
    }
}

#[test]
fn i32_widening_source_rejects_nonexact_inputs_arity_and_type_arguments() {
    for expr in [
        "i64_from_i32()",
        "i64_from_i32(i32_from_i64(1),i32_from_i64(2))",
        "i64_from_i32(true)",
        "i64_from_i32(1)",
        "i64_from_i32(1.0)",
        "i64_from_i32<i32>(i32_from_i64(1))",
        "i64_from_i32(i64_from_i32(i32_from_i64(1)))",
    ] {
        assert!(
            parse_nuis_module(&format!("mod cpu Main{{fn main()->i64{{return {expr};}}}}"))
                .is_err(),
            "{expr}"
        );
    }
    for ty in ["i64", "f32", "f64", "bool", "ref i32", "i32?", "i32<i64>"] {
        assert!(parse_nuis_module(&format!("mod cpu Main{{fn wrong(value:{ty})->i64{{return i64_from_i32(value);}}fn main()->i64{{return 0;}}}}" )).is_err(),"{ty}");
    }
}

#[test]
fn i32_widening_source_generic_arguments_and_lambda_local_inference_agree() {
    for value in [i32::MIN, -1, i32::MAX] {
        let source=format!("mod cpu Main{{fn keep<T>(value:T)->T{{return value;}}fn apply(value:i32,f:Fn1<i32,i64>)->i64{{return f(value);}}fn main()->i64{{let value=i32_from_i64({value});let a=keep(i64_from_i32(value));let b=apply(value,|x:i32|{{let wide=i64_from_i32(x);return keep(wide);}});print(a);print(b);return 0;}}}}");
        assert_eq!(output(&source), [i64::from(value); 2]);
    }
}
