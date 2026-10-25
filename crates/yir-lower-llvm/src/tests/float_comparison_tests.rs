use super::support::*;
use crate::{emit_loop_compare, CpuLoopScalarKind};

#[test]
fn scalar_float_comparisons_preserve_ieee_nan_equality_rules() {
    for (constant, ty) in [("cpu.const_f32", "float"), ("cpu.const_f64", "double")] {
        for (op, predicate) in [("cpu.ne", "une"), ("cpu.eq", "oeq")] {
            let mut module = module_with_cpu0();
            push_cpu_node(&mut module, "left", constant, vec!["NaN"]);
            push_cpu_node(&mut module, "right", constant, vec!["1.0"]);
            push_cpu_node(&mut module, "compared", op, vec!["left", "right"]);
            push_dep(&mut module, "left", "compared");
            push_dep(&mut module, "right", "compared");
            let llvm = emit_module(&module).unwrap();
            assert!(llvm.contains(&format!("fcmp {predicate} {ty}")), "{llvm}");
            assert!(!llvm.contains("fcmp one "));
        }
    }
}

#[test]
fn loop_float_comparisons_preserve_ieee_nan_equality_rules() {
    for (kind, ty) in [
        (CpuLoopScalarKind::F32, "float"),
        (CpuLoopScalarKind::F64, "double"),
    ] {
        for (op, predicate) in [
            ("eq", "oeq"),
            ("ne", "une"),
            ("lt", "olt"),
            ("le", "ole"),
            ("gt", "ogt"),
            ("ge", "oge"),
        ] {
            let mut body = Vec::new();
            emit_loop_compare(&mut body, &mut 0, kind, op, "%left", "%right").unwrap();
            assert!(body[0].contains(&format!("fcmp {predicate} {ty} %left, %right")));
        }
    }
    let mut body = Vec::new();
    emit_loop_compare(
        &mut body,
        &mut 0,
        CpuLoopScalarKind::I64,
        "ne",
        "%left",
        "%right",
    )
    .unwrap();
    assert!(body[0].contains("icmp ne i64"));
}
