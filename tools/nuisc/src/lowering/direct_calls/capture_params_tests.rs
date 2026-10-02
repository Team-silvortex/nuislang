use super::*;

fn leaves(kinds: &[&str]) -> Vec<NirParam> {
    kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| NirParam {
            name: format!("f{index}"),
            ty: NirTypeRef {
                name: (*kind).into(),
                generic_args: vec![],
                is_ref: false,
                is_optional: false,
            },
        })
        .collect()
}

#[test]
fn boolean_capture_groups_preserve_scalar_order_and_bound_word_width() {
    assert!(CapturePlan::new(leaves(&["i64", "bool", "f64"])).is_none());
    let plan = CapturePlan::new(leaves(&["bool", "i32", "bool", "f32", "f64"])).unwrap();
    assert_eq!(
        plan.slots,
        [
            Slot::Bools(vec![0, 2]),
            Slot::Scalar(1),
            Slot::Scalar(3),
            Slot::Scalar(4)
        ]
    );
    for count in [2, 62, 63, 64, 65, 126, 127, 128] {
        let plan = CapturePlan::new(leaves(&vec!["bool"; count])).unwrap();
        assert_eq!(plan.slots.len(), count.div_ceil(BOOLS_PER_WORD));
        let flattened = plan
            .slots
            .iter()
            .flat_map(|slot| match slot {
                Slot::Scalar(index) => vec![*index],
                Slot::Bools(group) => {
                    assert!((2..=BOOLS_PER_WORD).contains(&group.len()));
                    group.clone()
                }
                Slot::Record { .. } => panic!("boolean-only plan must not introduce records"),
            })
            .collect::<Vec<_>>();
        assert_eq!(flattened, (0..count).collect::<Vec<_>>());
    }
}

#[test]
fn scoped_record_plans_keep_boolean_slots_and_unproven_records_independent() {
    let fields = (0..60)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let module = crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{ struct Carry {{ {fields} }}
            struct Fixed {{ x: bool, y: bool, z: i64 }}
            fn work(state: Carry, fixed: Fixed, flag: bool, index: i64) -> i64 {{ return index; }} }}"
    )).unwrap();
    let function = &module.functions[0];
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let flat = function
        .params
        .iter()
        .flat_map(|p| {
            crate::lowering::scalar_record_shape::Shape::from_definitions(&p.ty, &definitions)
                .unwrap()
                .leaves()
                .into_iter()
                .map(|(path, ty)| NirParam {
                    name: if path.is_empty() {
                        p.name.clone()
                    } else {
                        format!("{}.{}", p.name, path.join("."))
                    },
                    ty,
                })
        })
        .collect::<Vec<_>>();
    assert_eq!(flat.len(), 65);
    let ordinary = CapturePlan::for_generated(flat.clone(), function, &module).unwrap();
    assert!(ordinary.slots.len() <= 64);
    assert!(ordinary.slots.iter().any(|s| matches!(s, Slot::Bools(_))));
    assert!(!ordinary.supports_scoped(function, &BTreeMap::new()));
    assert!(CapturePlan::for_scoped(flat.clone(), function, &module, &BTreeMap::new()).is_none());
    let allowed = BTreeSet::from([0]);
    let plan = CapturePlan::records(flat.clone(), function, &module, Some(&allowed)).unwrap();
    assert_eq!(plan.slots.len(), 6);
    assert!(
        matches!(&plan.slots[0], Slot::Record { parameter, leaves, .. } if parameter.name == "state" && *leaves == (0..60))
    );
    assert_eq!(
        &plan.slots[1..],
        &(60..65).map(Slot::Scalar).collect::<Vec<_>>()
    );
    let mut malformed = flat;
    malformed[60].name = "state.f0".into();
    assert!(CapturePlan::records(malformed, function, &module, Some(&allowed)).is_none());
}

#[test]
fn generated_capture_transport_does_not_rewrite_user_signatures_or_namesakes() {
    let source = "mod cpu Main {
        struct Pair { a: bool, b: bool }
        @noinline fn __nuis_conditional_value(a: bool, b: bool) -> bool { return a != b; }
        @noinline fn relay(value: Pair) -> Pair { return value; }
        @noinline fn choose(a: bool, b: bool) -> Pair {
            let value = Pair { a: a, b: b };
            if __nuis_conditional_value(a, b) { let value = relay(value); }
            else { let value = Pair { a: value.b, b: value.a }; }
            return value;
        }
        fn main() -> i64 { let value = choose(true, false); print(value.a); print(value.b); return 0; }
    }";
    let yir = crate::pipeline::compile_source(source).unwrap().yir;
    for name in ["__nuis_conditional_value", "choose", "relay"] {
        let function = yir.functions.iter().find(|f| f.name == name).unwrap();
        assert_eq!(function.parameters.len(), 2);
        assert!(function.parameters.iter().all(|p| p.ty == "bool"));
    }
    let generated = yir
        .functions
        .iter()
        .find(|f| {
            !["__nuis_conditional_value", "choose", "relay", "main"].contains(&f.name.as_str())
                && f.parameters.len() == 1
                && f.parameters[0].ty == "i64"
        })
        .unwrap();
    assert_eq!(generated.parameters.len(), 1);
    assert_eq!(generated.parameters[0].ty, "i64");
    yir_verify::verify_module(&yir).unwrap();
    yir_lower_llvm::emit_module(&yir).unwrap();
}

#[test]
fn scoped_boolean_iteration_parameters_remain_independent_of_value_captures() {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        @noinline fn walk(limit: i64, first: bool, second: bool) -> bool {
            let i = 0; let a = first; let b = second;
            while i < limit { let i = i + 1; let a = a == false; let b = a; }
            return a;
        }
        fn main() -> i64 { print(walk(3, true, false)); return 0; }
    }",
    )
    .unwrap();
    let mut module = module;
    let outlined = crate::lowering::buffer_loop_outline::outline_buffer_loops(&mut module).unwrap();
    let iterations = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_scalar_iteration"))
        .collect::<Vec<_>>();
    assert!(!iterations.is_empty());
    for function in iterations {
        assert!(!outlined.capture_plans.contains_key(&function.name));
    }
}

#[test]
fn generated_mixed_value_iterations_keep_word_maps_independent_of_boolean_compaction() {
    let source = "mod cpu Main {
        struct Pair { a: bool, b: bool }
        fn relay(value: Pair) -> Pair { return value; }
        fn walk(value: Pair, flag: bool) -> Pair {
            let carry = value; let i = 0;
            while i < 2 {
                if flag { let carry = relay(carry); }
                else { let carry = Pair { a: carry.b, b: carry.a }; }
                let i = i + 1;
            }
            return carry;
        }
        fn main() -> i64 { return 0; }
    }";
    let mut module = crate::frontend::parse_nuis_module(source).unwrap();
    let outlined = crate::lowering::buffer_loop_outline::outline_buffer_loops(&mut module).unwrap();
    let selection = module
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_scalar_iteration"))
        .unwrap();
    let eligible = BTreeSet::from([selection.name.as_str()]);
    assert!(
        scoped_loop_lowering::collect_scoped_call_targets(&module, &eligible)
            .contains(&selection.name)
    );
    assert!(!outlined.capture_plans.contains_key(&selection.name));
    assert!(selection.params.iter().any(|p| p.ty.name == "bool"));
    assert!(selection
        .params
        .iter()
        .any(|p| module.structs.iter().any(|definition| {
            definition.name == p.ty.name
                && definition.fields.len() == 2
                && definition.fields.iter().all(|f| f.ty.name == "i64")
        })));
}

#[test]
fn generated_record_plan_is_bounded_and_only_needed_after_scalar_compaction() {
    for (count, kind, records) in [
        (63, "i64", false),
        (64, "i64", true),
        (64, "bool", false),
        (65, "i64", false),
    ] {
        let fields = (0..count)
            .map(|i| format!("f{i}: {kind}"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!("mod cpu Main {{ struct State {{ {fields} }} fn choose(value: State, flag: bool) -> State {{ return value; }} }}");
        let module = crate::frontend::parse_nuis_module(&source).unwrap();
        let function = module
            .functions
            .iter()
            .find(|f| f.name == "choose")
            .unwrap();
        let mut flattened = leaves(&vec![kind; count]);
        for leaf in &mut flattened {
            leaf.name = format!("value.{}", leaf.name);
        }
        flattened.push(function.params[1].clone());
        let plan = CapturePlan::for_generated(flattened, function, &module);
        assert_eq!(
            plan.as_ref().is_some_and(|plan| plan
                .slots
                .iter()
                .any(|slot| matches!(slot, Slot::Record { .. }))),
            records
        );
        if records {
            assert_eq!(plan.unwrap().slots.len(), 2);
        }
    }
}

#[test]
fn scoped_record_plan_groups_exact_leaf_ranges_between_scalar_inputs() {
    let fields = (0..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let module = crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{ struct State {{ {fields} }} fn helper(index: i64, value: State, limit: i64) -> State {{ return value; }} }}"
    )).unwrap();
    let function = &module.functions[0];
    let mut flattened = vec![function.params[0].clone()];
    let mut fields = leaves(&["i64"; 64]);
    for field in &mut fields {
        field.name = format!("value.{}", field.name);
    }
    flattened.extend(fields);
    flattened.push(function.params[2].clone());
    let plan = CapturePlan::for_generated(flattened, function, &module).unwrap();
    let record = (0..64)
        .map(|i| format!("$owned_struct_carry:{i}:seed{i}"))
        .collect::<Vec<_>>();
    let mut input = vec!["$current".into()];
    input.extend(record.clone());
    input.push("limit".into());
    let mapped = plan.lower_scoped_arguments(&input).unwrap();
    assert_eq!(mapped.len(), 3);
    assert_eq!(mapped[0], "$current");
    assert_eq!(mapped[2], "limit");
    assert_eq!(
        yir_core::loop_carry_contract::ScopedRecordInput::parse(&mapped[1])
            .unwrap()
            .unwrap()
            .operands,
        record
    );
    assert!(plan.lower_scoped_arguments(&input[..65]).is_err());
    assert!(!plan.supports_scoped(function, &BTreeMap::new()));
    assert!(CapturePlan::new(leaves(&["bool", "bool"]))
        .unwrap()
        .lower_scoped_arguments(&["first".into(), "second".into()])
        .is_err());
}

#[test]
fn wide_record_scoped_helpers_keep_carry_and_seed_signature_mapping() {
    let fields = (0..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..64)
        .map(|i| format!("f{i}: carry.f{i} + 1"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "mod cpu Main {{
        struct State {{ {fields} }}
        @noinline fn relay(value: State) -> State {{ return value; }}
        fn walk(value: State, flag: bool) -> State {{
            let carry = value; let i = 0;
            while i < 2 {{
                if flag {{ let carry = relay(carry); }}
                else {{ let carry = State {{ {values} }}; }}
                let i = i + 1;
            }}
            return carry;
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    );
    let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
    let outlined = crate::lowering::buffer_loop_outline::outline_buffer_loops(&mut module).unwrap();
    let eligible = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_"))
        .map(|f| f.name.as_str())
        .collect();
    let scoped = scoped_loop_lowering::collect_scoped_call_targets(&module, &eligible);
    assert!(
        scoped.iter().any(|name| {
            module
                .functions
                .iter()
                .find(|function| function.name == *name)
                .unwrap()
                .params
                .iter()
                .any(|parameter| parameter.ty.name == "State")
        }),
        "fixture must carry a whole record through a scoped signature"
    );
    assert!(
        scoped
            .iter()
            .any(|name| outlined.capture_plans.contains_key(name)),
        "proven wide scoped record must use a mapped record input"
    );
}
