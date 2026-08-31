use super::*;
use yir_core::{Operation, RegisteredMod, ResourceKind, StructValue, VariantUnionValue};

fn resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn call(layout: &str) -> Node {
    Node {
        name: "owned_call".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "call_owned_struct".to_owned(),
            args: vec!["helper".to_owned(), layout.to_owned()],
        },
    }
}

#[test]
fn canonical_owned_struct_call_produces_default_fields() {
    let value = CpuMod
        .execute(
            &call("Summary{ready:bool;score:i64;ratio:f64}"),
            &resource(),
            &mut ExecutionState::default(),
        )
        .expect("execute canonical owned struct call");
    assert_eq!(
        value,
        Value::Struct(StructValue {
            type_name: "Summary".to_owned(),
            fields: vec![
                ("ready".to_owned(), Value::Bool(false)),
                ("score".to_owned(), Value::Int(0)),
                ("ratio".to_owned(), Value::F64(0.0)),
            ],
        })
    );
}

#[test]
fn canonical_owned_variant_call_produces_default_union() {
    let value = CpuMod
        .execute(
            &call(
                "__nuis_variant_union__Result{tag:i64;Result.Ok:Result.Ok{value:i64};Result.Err:Result.Err{message:String}}",
            ),
            &resource(),
            &mut ExecutionState::default(),
        )
        .expect("execute canonical owned variant call");
    let Value::VariantUnion(VariantUnionValue {
        parent_type_name,
        active_variant,
        variants,
    }) = value
    else {
        panic!("expected variant union")
    };
    assert_eq!(parent_type_name, "Result");
    assert_eq!(active_variant, "Result.Ok");
    assert_eq!(variants.len(), 2);
    assert_eq!(variants["Result.Ok"].fields[0].1, Value::Int(0));
}

#[test]
fn legacy_or_malformed_owned_struct_layout_fails_closed() {
    let error = CpuMod
        .execute(
            &call("Summary|ready:bool,score:i64"),
            &resource(),
            &mut ExecutionState::default(),
        )
        .expect_err("legacy layout must not bypass the canonical parser");
    assert!(error.contains("invalid owned struct layout"));
}

#[test]
fn owned_variant_return_preserves_the_union_shape() {
    let layout = "__nuis_variant_union__Result{tag:i64;Result.Ok:Result.Ok{value:i64};Result.Err:Result.Err{}}";
    let mut state = ExecutionState::default();
    let called = CpuMod
        .execute(&call(layout), &resource(), &mut state)
        .expect("create default variant union");
    state.values.insert("called".to_owned(), called.clone());
    let returned = CpuMod
        .execute(
            &Node {
                name: "owned_return".to_owned(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "return_owned_struct".to_owned(),
                    args: vec!["called".to_owned(), layout.to_owned()],
                },
            },
            &resource(),
            &mut state,
        )
        .expect("return owned variant union");
    assert_eq!(returned, called);
}
