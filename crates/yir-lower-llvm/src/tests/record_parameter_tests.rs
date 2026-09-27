use super::support::*;
use crate::call_parameters::CpuCallParameterKind;
use crate::native_session::value_transport::NativeValueLayout;
use crate::LlvmValueRef;
use yir_core::{
    YirFunction, YirFunctionParameter, YirFunctionResult, YirFunctionRole, YirValueOwnership,
};

fn module() -> YirModule {
    let mut module = module_with_cpu0();
    push_cpu_const_i64(&mut module, "seed", "6");
    push_cpu_node(
        &mut module,
        "record",
        "cpu.struct",
        vec!["State", "value=seed"],
    );
    push_cpu_node(
        &mut module,
        "invoke",
        "cpu.call_i64",
        vec!["read", "record"],
    );
    push_cpu_node(
        &mut module,
        "input",
        "cpu.param_value_struct",
        vec!["0", "State{value:i64}"],
    );
    push_cpu_node(&mut module, "field", "cpu.field", vec!["input", "value"]);
    push_cpu_node(&mut module, "result", "cpu.return_i64", vec!["field"]);
    push_deps(
        &mut module,
        &[
            ("seed", "record"),
            ("record", "invoke"),
            ("input", "field"),
            ("field", "result"),
        ],
    );
    for name in ["input", "field", "result"] {
        module.node_lanes.insert(name.into(), "fn:read".into());
    }
    module.functions.push(YirFunction {
        name: "read".into(),
        domain: "cpu".into(),
        role: YirFunctionRole::Helper,
        parameters: vec![YirFunctionParameter {
            name: "state".into(),
            ty: "State".into(),
            ownership: YirValueOwnership::Value,
            node: "input".into(),
        }],
        result: Some(YirFunctionResult {
            ty: "i64".into(),
            ownership: YirValueOwnership::Value,
            node: "result".into(),
        }),
        body_nodes: vec!["input".into(), "field".into(), "result".into()],
    });
    module
}

#[test]
fn record_parameters_use_value_signatures_without_task_invokers() {
    let llvm = emit_module(&module()).unwrap();
    assert!(llvm.contains("define i64 @nuis_fn_read([1 x i64] %arg0)"));
    assert!(llvm.contains("call i64 @nuis_fn_read([1 x i64]"));
    assert!(llvm.contains("extractvalue [1 x i64] %arg0, 0"));
    assert!(!llvm.contains("define i64 @nuis_task_invoker_read("));
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
}

#[test]
fn record_parameters_reject_deferred_task_transport() {
    let mut module = module();
    push_cpu_node(
        &mut module,
        "schedule",
        "cpu.async_call",
        vec!["read", "record"],
    );
    push_cpu_node(
        &mut module,
        "task",
        "cpu.spawn_task",
        vec!["read", "invoke"],
    );
    push_deps(&mut module, &[("record", "schedule"), ("invoke", "task")]);
    module.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: "schedule".into(),
        to: "invoke".into(),
    });
    let error = emit_module(&module).unwrap_err();
    assert!(
        error.contains("record parameters cannot escape into task thunks"),
        "{error}"
    );
}

#[test]
fn record_parameters_cannot_be_coerced_into_scoped_scalar_arguments() {
    let kinds = [CpuCallParameterKind::Record(
        NativeValueLayout::parse("State{value:i64}").unwrap(),
    )];
    let registers = BTreeMap::from([(
        "record".into(),
        LlvmValueRef::Struct(crate::StructLlvmValueRef {
            type_name: "State".into(),
            fields: vec![("value".into(), LlvmValueRef::I64("6".into()))],
        }),
    )]);
    assert!(crate::call_lowering::lower_branch_scalar_args(
        &registers,
        &BTreeMap::new(),
        &["record".into()],
        &kinds,
        &[None],
    )
    .is_none());
}

#[test]
fn all_record_arguments_are_checked_before_any_call_packing() {
    let layout = NativeValueLayout::parse("State{value:i64}").unwrap();
    let signatures = BTreeMap::from([(
        "read".into(),
        crate::CpuHelperSignature {
            params: vec![
                CpuCallParameterKind::Record(layout.clone()),
                CpuCallParameterKind::Record(layout),
            ],
            implicit_parameters: vec![],
            mutex_permit_params: vec![None, None],
            ret: crate::CpuCallScalarKind::I64,
            owned_struct_return: false,
            owned_struct_layout: None,
            native_value_return: None,
            owned_external_buffer_return: None,
        },
    )]);
    let value = |name: &str| {
        LlvmValueRef::Struct(crate::StructLlvmValueRef {
            type_name: name.into(),
            fields: vec![("value".into(), LlvmValueRef::I64("6".into()))],
        })
    };
    let mut registers = BTreeMap::from([
        ("left".into(), value("State")),
        ("right".into(), value("Other")),
    ]);
    let node = Node {
        name: "invoke".into(),
        resource: "cpu0".into(),
        op: Operation::parse(
            "cpu.call_i64",
            vec!["read".into(), "left".into(), "right".into()],
        )
        .unwrap(),
    };
    let mut body = vec!["entry:".into()];
    let mut next_reg = 73;
    let mut last = Some("previous".into());
    assert!(crate::call_lowering::lower_cpu_call_node(
        &node,
        &mut body,
        &mut registers,
        &signatures,
        &mut BTreeMap::new(),
        &Default::default(),
        &mut next_reg,
        &mut last
    )
    .is_err());
    assert_eq!(body, ["entry:"]);
    assert_eq!(next_reg, 73);
    assert_eq!(last.as_deref(), Some("previous"));
    assert!(!registers.contains_key("invoke"));
}
