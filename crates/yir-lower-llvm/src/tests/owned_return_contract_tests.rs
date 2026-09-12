use super::support::*;
use crate::{LlvmValueRef, StructLlvmValueRef};

fn lower_guard(value: LlvmValueRef, local_layout: Option<&str>) -> Result<String, String> {
    let layout = yir_core::parse_owned_struct_layout("State{carry0:i64;carry1:i64}").unwrap();
    let mut args = vec!["condition".to_owned(), "returned".to_owned()];
    args.extend(local_layout.map(str::to_owned));
    let node = Node {
        name: "early".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse("cpu.guard_return", args).unwrap(),
    };
    let registers = BTreeMap::from([
        ("condition".to_owned(), LlvmValueRef::I64("1".to_owned())),
        ("returned".to_owned(), value),
    ]);
    let mut body = Vec::new();
    crate::guard_return_lowering::lower_cpu_guard_return_node(
        &node,
        &mut body,
        &registers,
        &mut 0,
        &mut 0,
        CpuCallScalarKind::I64,
        Some(&layout),
    )?;
    Ok(body.join("\n"))
}

fn returned() -> StructLlvmValueRef {
    StructLlvmValueRef {
        type_name: "State".to_owned(),
        fields: vec![
            ("carry0".to_owned(), LlvmValueRef::I64("11".to_owned())),
            ("carry1".to_owned(), LlvmValueRef::I64("22".to_owned())),
        ],
    }
}

#[test]
fn aggregate_early_returns_enforce_the_function_layout_before_emitting() {
    let mut wrong_type = returned();
    wrong_type.type_name = "Other".to_owned();
    let mut short = returned();
    short.fields.pop();
    let mut wrong_scalar = returned();
    wrong_scalar.fields[1].1 = LlvmValueRef::I32("22".to_owned());
    let mut duplicate = returned();
    duplicate.fields[1].0 = "carry0".to_owned();
    for value in [
        LlvmValueRef::I64("1".to_owned()),
        LlvmValueRef::Struct(wrong_type),
        LlvmValueRef::Struct(short),
        LlvmValueRef::Struct(wrong_scalar),
        LlvmValueRef::Struct(duplicate),
    ] {
        assert!(lower_guard(value, None)
            .unwrap_err()
            .contains("function return layout"));
    }
    assert!(lower_guard(
        LlvmValueRef::Struct(returned()),
        Some("Other{carry0:i64;carry1:i64}")
    )
    .unwrap_err()
    .contains("function return layout"));
}

#[test]
fn aggregate_early_returns_pack_in_declared_field_order() {
    let mut value = returned();
    value.fields.reverse();
    let body = lower_guard(LlvmValueRef::Struct(value), None).unwrap();
    assert!(body.contains("i64 0, i64 11)"), "{body}");
    assert!(body.contains("i64 1, i64 22)"), "{body}");
    assert!(body.contains("ret i64"));
}

#[test]
fn declared_entry_result_wins_over_last_scheduled_scalar() {
    for reversed in [false, true] {
        let mut module = module_with_cpu0();
        push_cpu_node(&mut module, "answer", "cpu.const_i64", vec!["123"]);
        push_cpu_node(&mut module, "induction", "cpu.const_i64", vec!["8"]);
        module.functions.push(yir_core::YirFunction {
            name: "main".to_owned(),
            domain: "cpu".to_owned(),
            role: yir_core::YirFunctionRole::Entry,
            parameters: vec![],
            result: Some(yir_core::YirFunctionResult {
                ty: "i64".to_owned(),
                ownership: yir_core::YirValueOwnership::Value,
                node: "answer".to_owned(),
            }),
            body_nodes: vec!["answer".to_owned(), "induction".to_owned()],
        });
        if reversed {
            module.nodes.reverse();
            module.functions[0].body_nodes.reverse();
        }
        let llvm = emit_module(&module).unwrap();
        let answer = llvm
            .lines()
            .find_map(|line| line.trim().strip_suffix(" = add i64 0, 123"))
            .unwrap();
        let unused = llvm
            .lines()
            .find_map(|line| line.trim().strip_suffix(" = add i64 0, 8"))
            .unwrap();
        assert!(llvm.contains(&format!("ret i64 {answer}\n")), "{llvm}");
        assert!(!llvm.contains(&format!("ret i64 {unused}\n")));
    }
}

#[test]
fn unavailable_declared_entry_result_traps_instead_of_using_an_unrelated_scalar() {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "unrelated", "8");
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect();
    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect();
    for registers in [
        BTreeMap::new(),
        BTreeMap::from([("result".to_owned(), LlvmValueRef::Struct(returned()))]),
    ] {
        let emitted = emit_cpu_function(
            &nodes,
            &resources,
            &["unrelated".to_owned()],
            &registers,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &crate::default_branch_effect_llvm_emitters(),
            CpuCallScalarKind::I64,
            None,
            Some("result"),
            false,
            &mut 0,
        )
        .unwrap();
        assert!(emitted
            .body
            .contains("deferred lowering for declared CPU function result `result`"));
        assert!(
            emitted
                .body
                .ends_with("call void @llvm.trap()\n  unreachable"),
            "{}",
            emitted.body
        );
        assert!(!emitted.body.contains("ret i64"), "{}", emitted.body);
    }
}
