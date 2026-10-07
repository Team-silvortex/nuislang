use super::*;
use crate::frontend::parse_nuis_module;

fn module() -> NirModule {
    parse_nuis_module(
        "mod cpu Main {
        struct Leaf { flag: bool, tag: i32, value: i64, gain: f32, scale: f64 }
        struct Packet { payload: Leaf }
        struct Flat { value: i64 }
        fn leaf(value: Packet) -> Packet { return value; }
        fn checked(value: i64, divisor: i64) -> i32 { return i32_from_i64(value / divisor); }
        fn count(limit: i64) -> i64 {
            let index: i64 = 0;
            while index < limit { let index: i64 = index + 1; }
            return index;
        }
        fn wrapped(limit: i64, value: Packet) -> Packet {
            let work: i64 = count(limit); return leaf(value);
        }
        fn mixed_loop(limit: i64, value: Packet) -> Packet {
            let current = value;
            let index: i64 = 0;
            while index < limit {
                let current = leaf(current);
                let index: i64 = index + 1;
            }
            return current;
        }
        fn effect(value: Packet) -> Packet { print(value.payload.value); return value; }
        fn caller(value: Packet) -> Packet { return effect(value); }
        fn cycle_a(value: Packet) -> Packet { return cycle_b(value); }
        fn cycle_b(value: Packet) -> Packet { return cycle_a(value); }
        fn main() -> i64 { return 0; }
    }",
    )
    .unwrap()
}

#[test]
fn typed_values_reuse_nested_loop_dependencies_without_admitting_effects_or_cycles() {
    let module = module();
    let flat = layouts(&module);
    let loops = scalar_helpers::collect_with_layouts(&module, &flat);
    let typed = TypedLayouts::collect(&module);
    let values = scalar_helpers::collect_typed_values(&module, &typed, &loops);
    assert_eq!(
        values.keys().map(String::as_str).collect::<Vec<_>>(),
        ["checked", "count", "leaf", "main", "mixed_loop", "wrapped"]
    );
    assert!(values["wrapped"].may_loop);
    assert!(loops.contains_key("leaf"));
    assert!(values["mixed_loop"].may_loop);
    assert!(supported_type(&scalar_type("Packet"), &flat));
    let mut reversed = module.clone();
    reversed.structs.reverse();
    reversed.functions.reverse();
    let typed_reversed = TypedLayouts::collect(&reversed);
    let reordered = scalar_helpers::collect_typed_values(&reversed, &typed_reversed, &loops);
    assert_eq!(
        values.keys().collect::<Vec<_>>(),
        reordered.keys().collect::<Vec<_>>()
    );
    let zero = zero_value(&scalar_type("Packet"), &typed);
    assert_eq!(
        value_type(&zero, &Scope::new(), &values, &typed),
        Some(scalar_type("Packet"))
    );
    assert_eq!(
        value_type(&zero, &Scope::new(), &values, &flat),
        Some(scalar_type("Packet"))
    );
    let mut inputs = BTreeSet::new();
    collect_inputs(&zero, &mut inputs);
    assert!(inputs.is_empty());
}

#[test]
fn nested_value_admission_does_not_grant_nonloop_control_outlining_authority() {
    let module = module();
    let layouts = layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let roots = scalar_helpers::control_roots(&module, &layouts, &catalog);
    assert!(roots.contains("count"));
    assert!(roots.contains("mixed_loop"));
    assert!(catalog["wrapped"].may_loop);
    for name in ["leaf", "wrapped", "effect", "caller", "cycle_a", "cycle_b"] {
        assert!(!roots.contains(name), "{name}");
    }
    let mut reversed = module.clone();
    reversed.functions.reverse();
    reversed.structs.reverse();
    assert_eq!(
        roots,
        scalar_helpers::control_roots(&reversed, &layouts, &catalog)
    );
}

#[test]
fn flat_control_interfaces_admit_completed_nested_results_without_nested_transport_authority() {
    let module = parse_nuis_module(
        "mod cpu Main {
        struct Input { value: i64, divisor: i64 }
        struct Selected { value: i64 }
        struct Envelope { selected: Selected, ignored: i64 }
        fn checked(value: i64) -> i64 { return 10 / value; }
        fn produce(input: Input) -> Envelope {
            return Envelope { selected: Selected { value: input.value },
                ignored: checked(input.divisor) };
        }
        fn choose(flag: bool, input: Input) -> i64 {
            if flag { let saved = produce(input).selected; return saved.value; }
            return checked(input.divisor);
        }
        fn nested_input(value: Envelope) -> i64 { return value.selected.value; }
        fn nested_return(input: Input) -> Envelope { return produce(input); }
        fn effect(input: Input) -> Envelope { print(input.value); return produce(input); }
        fn caller(input: Input) -> i64 { let saved = effect(input).selected; return saved.value; }
        fn cycle_a(input: Input) -> Envelope { return cycle_b(input); }
        fn cycle_b(input: Input) -> Envelope { return cycle_a(input); }
        fn cycle_caller(input: Input) -> i64 { let saved = cycle_a(input).selected; return saved.value; }
        fn main() -> i64 { return 0; }
    }",
    )
    .unwrap();
    for reversed in [false, true] {
        let mut module = module.clone();
        if reversed {
            module.functions.reverse();
            module.structs.reverse();
        }
        let layouts = layouts(&module);
        let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
        let roots = scalar_helpers::control_roots(&module, &layouts, &catalog);
        assert!(roots.contains("choose"));
        for name in [
            "produce",
            "nested_input",
            "nested_return",
            "effect",
            "caller",
            "cycle_a",
            "cycle_b",
            "cycle_caller",
        ] {
            assert!(!roots.contains(name), "{name}");
        }
    }
}

#[test]
fn typed_layouts_reject_cycles_resources_empty_and_duplicate_fields_transitively() {
    for mutation in ["cycle", "resource", "empty", "duplicate", "generic"] {
        let mut module = module();
        let leaf = module
            .structs
            .iter_mut()
            .find(|d| d.name == "Leaf")
            .unwrap();
        match mutation {
            "cycle" => leaf.fields[0].ty = scalar_type("Packet"),
            "resource" => leaf.fields[0].ty = scalar_type("Bytes"),
            "empty" => leaf.fields.clear(),
            "duplicate" => leaf.fields[1].name = leaf.fields[0].name.clone(),
            "generic" => leaf.fields[0].ty.generic_args.push(scalar_type("i64")),
            _ => unreachable!(),
        }
        let typed = TypedLayouts::collect(&module);
        assert!(!supported_type(&scalar_type("Leaf"), &typed), "{mutation}");
        assert!(
            !supported_type(&scalar_type("Packet"), &typed),
            "{mutation}"
        );
        assert!(supported_type(&scalar_type("Flat"), &typed));
    }
}

#[test]
fn typed_constructors_require_exact_leaf_types_unique_fields_and_nominal_records() {
    let module = module();
    let typed = TypedLayouts::collect(&module);
    for mutation in ["kind", "duplicate", "missing", "nominal", "generic"] {
        let mut value = zero_value(&scalar_type("Leaf"), &typed);
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut value
        else {
            unreachable!()
        };
        match mutation {
            "kind" => fields[1].1 = NirExpr::Int(0),
            "duplicate" => fields[1].0 = fields[0].0.clone(),
            "missing" => {
                fields.pop();
            }
            "nominal" => *type_name = "Flat".into(),
            "generic" => type_args.push(scalar_type("i64")),
            _ => unreachable!(),
        }
        assert!(
            value_type(&value, &Scope::new(), &ScalarHelpers::new(), &typed).is_none(),
            "{mutation}"
        );
    }
    assert_eq!(
        binary_type(NirBinaryOp::Add, scalar_type("f32"), scalar_type("f32")),
        Some(scalar_type("f32"))
    );
    assert!(binary_type(NirBinaryOp::Div, scalar_type("f32"), scalar_type("f32")).is_none());
    for op in [NirBinaryOp::Add, NirBinaryOp::Sub, NirBinaryOp::Mul] {
        assert_eq!(
            binary_type(op, scalar_type("f64"), scalar_type("f64")),
            Some(scalar_type("f64"))
        );
    }
    for op in [
        NirBinaryOp::Div,
        NirBinaryOp::Rem,
        NirBinaryOp::Eq,
        NirBinaryOp::Lt,
    ] {
        assert!(binary_type(op, scalar_type("f64"), scalar_type("f64")).is_none());
    }
    assert!(binary_type(NirBinaryOp::Add, scalar_type("f32"), scalar_type("f64")).is_none());
}

#[test]
fn typed_layouts_bound_neutral_initializer_expansion_before_materializing_a_type_dag() {
    for (width, levels, accepted, rejected) in [(2, 20, 8, 12), (1, 70, 63, 64)] {
        let mut module = module();
        let template = module
            .structs
            .iter()
            .find(|d| d.name == "Leaf")
            .unwrap()
            .clone();
        for level in 0..levels {
            let mut definition = template.clone();
            definition.name = format!("R{level}");
            definition.fields = (0..width)
                .map(|field| {
                    let mut entry = template.fields[0].clone();
                    entry.name = format!("f{field}");
                    entry.ty = if level == 0 {
                        scalar_type("bool")
                    } else {
                        scalar_type(&format!("R{}", level - 1))
                    };
                    entry
                })
                .collect();
            module.structs.push(definition);
        }
        module.structs.reverse();
        let layouts = TypedLayouts::collect(&module);
        assert!(supported_type(
            &scalar_type(&format!("R{accepted}")),
            &layouts
        ));
        assert!(!supported_type(
            &scalar_type(&format!("R{rejected}")),
            &layouts
        ));
        assert!(!supported_type(
            &scalar_type(&format!("R{}", levels - 1)),
            &layouts
        ));
    }
}

#[test]
fn unary_float_negation_sign_words_admit_exact_i64_xor_without_profile_widening() {
    for ty in ["i64", "bool"] {
        assert_eq!(
            binary_type(NirBinaryOp::Xor, scalar_type(ty), scalar_type(ty)),
            Some(scalar_type(ty))
        );
    }
    for ty in ["i32", "f32", "f64", "Bytes"] {
        assert!(binary_type(NirBinaryOp::Xor, scalar_type(ty), scalar_type(ty)).is_none());
    }
    for mutation in ["reference", "optional", "generic"] {
        let mut ty = scalar_type("i64");
        match mutation {
            "reference" => ty.is_ref = true,
            "optional" => ty.is_optional = true,
            "generic" => ty.generic_args.push(scalar_type("bool")),
            _ => unreachable!(),
        }
        assert!(binary_type(NirBinaryOp::Xor, ty.clone(), ty).is_none());
    }
    assert!(binary_type(NirBinaryOp::Xor, scalar_type("i64"), scalar_type("bool")).is_none());
    let module = parse_nuis_module(
        "mod cpu Main {
        fn negative32(value: f32) -> f32 { return -relay32(value); }
        fn negative64(value: f64) -> f64 { return -relay64(value); }
        fn relay32(value: f32) -> f32 { return value; }
        fn relay64(value: f64) -> f64 { return value; }
        fn cycle_a(value: f64) -> f64 { return -cycle_b(value); }
        fn cycle_b(value: f64) -> f64 { return -cycle_a(value); }
        fn effect(value: f64) -> f64 { print(9); return value; }
        fn effect_caller(value: f64) -> f64 { return -effect(value); }
        fn main() -> i64 { return 0; }
    }",
    )
    .unwrap();
    let flat = layouts(&module);
    let loops = scalar_helpers::collect_with_layouts(&module, &flat);
    let typed = TypedLayouts::collect(&module);
    let values = scalar_helpers::collect_typed_values(&module, &typed, &loops);
    for name in ["cycle_a", "cycle_b", "effect", "effect_caller"] {
        assert!(!values.contains_key(name), "{name}");
    }
    let mut reversed = module.clone();
    reversed.functions.reverse();
    let reordered = scalar_helpers::collect_typed_values(&reversed, &typed, &loops);
    assert_eq!(
        values.keys().collect::<Vec<_>>(),
        reordered.keys().collect::<Vec<_>>()
    );
    for (name, ty) in [("negative32", "f32"), ("negative64", "f64")] {
        assert!(values.contains_key(name));
        let function = module.functions.iter().find(|f| f.name == name).unwrap();
        assert!(scalar_helpers::contains_calls(&function.body));
        let NirStmt::Return(Some(expr)) = &function.body[0] else {
            panic!()
        };
        let scope = Scope::from([("value".into(), scalar_type(ty))]);
        assert_eq!(
            value_type(expr, &scope, &values, &typed),
            Some(scalar_type(ty))
        );
        assert_eq!(
            value_type(expr, &scope, &values, &flat),
            Some(scalar_type(ty))
        );
        let mut wrong = scope.clone();
        wrong.insert(
            "value".into(),
            scalar_type(if ty == "f32" { "f64" } else { "f32" }),
        );
        assert!(value_type(expr, &wrong, &values, &typed).is_none());
        let mut inputs = BTreeSet::new();
        collect_inputs(expr, &mut inputs);
        assert_eq!(inputs, BTreeSet::from(["value".into()]));
    }
}

#[test]
fn unary_float_negation_word_wrappers_keep_aggregate_iteration_classification() {
    let module = parse_nuis_module(
        "mod cpu Main {
        struct State { value: f32 }
        fn compute(state: State) -> f32 { let value: f32 = -state.value; return value; }
        fn main() -> i64 { return 0; }
    }",
    )
    .unwrap();
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "compute")
        .unwrap();
    assert!(has_aggregate_expressions(&function.body));
    assert!(!has_aggregate_expressions(&[NirStmt::Let {
        name: "value".into(),
        ty: Some(scalar_type("f32")),
        value: NirExpr::UnpackF32Word(Box::new(NirExpr::Int(0))),
    }]));
}
